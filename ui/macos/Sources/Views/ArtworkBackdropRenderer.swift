import AppKit
import MetalKit
import MetalPerformanceShaders
import simd

enum BackdropError: Error {
    case unavailable, allocation, shaderFunction, encoder
}

// Music.app's own default.metallib ABI, not MediaCoreUI's identically named shaders.
struct BackdropModel {
    var matrix = matrix_identity_float4x4
    var timeScale: Float = 60
    var padding: (Float, Float, Float) = (0, 0, 0)
}

struct BackdropUniforms {
    var viewMatrix = matrix_identity_float4x4
    var time: Float = 0                         // 0x40
    var textureTransitionMix: Float = 0         // 0x44: old artwork weight
    var meshWarpTimeScale: Float = 1.75
    var saturation: Float = 1.3
    var whiteScrimAlpha: Float = 0
    var blackScrimAlpha: Float = 0.25
    var factorForDarkMode: Float = 0.2
    var factorForLightMode: Float = 0
    var floor: Float = 0.07
    var ceiling: Float = 0.97
    var padding: (Float, Float, Float, Float, Float, Float) = (0, 0, 0, 0, 0, 0)
    var model0 = BackdropModel()                // 0x80
    var model1 = BackdropModel()
    var model2 = BackdropModel()
}

struct BackdropVertex {
    var position: SIMD4<Float>
    var from: SIMD4<Float>
    var to: SIMD4<Float>
    var uv: SIMD2<Float>
}

final class ArtworkBackdropRenderer: NSObject, MTKViewDelegate {
    let device: MTLDevice
    let queue: MTLCommandQueue
    private let rotationPipeline: MTLRenderPipelineState
    private let pinchPipeline: MTLRenderPipelineState
    private let meshBuffer: MTLBuffer
    private let indexBuffer: MTLBuffer
    private let indexCount: Int
    private let loader: MTKTextureLoader
    private let fallback: MTLTexture
    private var image: NSImage?
    private var current: MTLTexture
    private var previous: MTLTexture?
    private var pending: MTLTexture?
    private var transitionStart: Double?
    private var rotationTexture: MTLTexture?
    private var blurredTexture: MTLTexture?
    private var blur: MPSImageGaussianBlur?
    private let epoch = CACurrentMediaTime()
    private var animationTime: Float = 0
    private var lastFrameTime: Double?
    var reduceMotion = false

    init(device: MTLDevice) throws {
        self.device = device
        guard MPSSupportsMTLDevice(device), let queue = device.makeCommandQueue() else {
            throw BackdropError.unavailable
        }
        self.queue = queue
        loader = MTKTextureLoader(device: device)
        let descriptor = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: .rgba8Unorm,
                                                                 width: 1, height: 1, mipmapped: false)
        descriptor.usage = .shaderRead
        guard let fallback = device.makeTexture(descriptor: descriptor) else { throw BackdropError.allocation }
        var pixel: [UInt8] = [28, 28, 36, 255]
        fallback.replace(region: MTLRegionMake2D(0, 0, 1, 1), mipmapLevel: 0,
                         withBytes: &pixel, bytesPerRow: 4)
        self.fallback = fallback
        current = fallback

        let mesh = Self.makeMesh()
        guard let vertices = mesh.vertices.withUnsafeBytes({ bytes in
            device.makeBuffer(bytes: bytes.baseAddress!, length: bytes.count)
        }), let indices = mesh.indices.withUnsafeBytes({ bytes in
            device.makeBuffer(bytes: bytes.baseAddress!, length: bytes.count)
        }) else { throw BackdropError.allocation }
        meshBuffer = vertices
        indexBuffer = indices
        indexCount = mesh.indices.count

        let library = try device.makeLibrary(source: Self.shaderSource, options: nil)
        func pipeline(_ vertex: String, _ fragment: String, mesh: Bool) throws -> MTLRenderPipelineState {
            let descriptor = MTLRenderPipelineDescriptor()
            guard let v = library.makeFunction(name: vertex), let f = library.makeFunction(name: fragment) else {
                throw BackdropError.shaderFunction
            }
            descriptor.vertexFunction = v
            descriptor.fragmentFunction = f
            descriptor.colorAttachments[0].pixelFormat = .bgra8Unorm
            if mesh {
                let layout = MTLVertexDescriptor()
                let offsets = [MemoryLayout<BackdropVertex>.offset(of: \.position)!,
                               MemoryLayout<BackdropVertex>.offset(of: \.from)!,
                               MemoryLayout<BackdropVertex>.offset(of: \.to)!,
                               MemoryLayout<BackdropVertex>.offset(of: \.uv)!]
                for i in 0..<4 {
                    layout.attributes[i].format = i == 3 ? .float2 : .float4
                    layout.attributes[i].offset = offsets[i]
                    layout.attributes[i].bufferIndex = 0
                }
                layout.layouts[0].stride = MemoryLayout<BackdropVertex>.stride
                descriptor.vertexDescriptor = layout
            }
            return try device.makeRenderPipelineState(descriptor: descriptor)
        }
        rotationPipeline = try pipeline("rotation_vertex", "rotation_fragment", mesh: false)
        pinchPipeline = try pipeline("pinch_vertex", "pinch_fragment", mesh: true)
        super.init()
    }

    func setImage(_ newImage: NSImage?) {
        guard image !== newImage else { return }
        do {
            var rect = NSRect(origin: .zero, size: newImage?.size ?? .zero)
            let texture: MTLTexture
            if let newImage {
                guard let cg = newImage.cgImage(forProposedRect: &rect, context: nil, hints: nil) else {
                    throw BackdropError.allocation
                }
                texture = try loader.newTexture(cgImage: cg, options: [.SRGB: false, .generateMipmaps: false])
            } else {
                texture = fallback
            }
            image = newImage
            pending = texture
        } catch {
            NSLog("Backdrop artwork upload failed: %@", String(describing: error))
        }
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        guard view.drawableSize.width > 0, view.drawableSize.height > 0,
              let drawable = view.currentDrawable, let command = queue.makeCommandBuffer() else { return }
        let now = CACurrentMediaTime() - epoch
        // Freeze while hidden; changing Reduce Motion never jumps animation phase.
        let delta = min(max(now - (lastFrameTime ?? now), 0), 1.0 / 15)
        animationTime += Float(delta) / (reduceMotion ? 10 : 1)
        lastFrameTime = now
        do {
            try encode(command: command, target: drawable.texture, time: animationTime, clock: now)
            command.present(drawable)
            command.commit()
        } catch {
            NSLog("Backdrop frame failed: %@", String(describing: error))
            view.isPaused = true
        }
    }

    // Shared by MTKView and offscreen GPU regression check.
    func encode(command: MTLCommandBuffer, target: MTLTexture, time: Float, clock: Double) throws {
        try resize(width: target.width, height: target.height)
        guard let rotationTexture, let blurredTexture, let blur else { throw BackdropError.allocation }
        if let start = transitionStart, clock - start >= 0.5 {
            previous = nil
            transitionStart = nil
        }
        if transitionStart == nil, let next = pending {
            previous = current
            current = next
            pending = nil
            transitionStart = clock
        }
        var uniforms = BackdropUniforms()
        uniforms.time = time
        uniforms.textureTransitionMix = transitionStart.map { Float(max(0, 1 - (clock - $0) / 0.5)) } ?? 0
        let aspect = Float(target.width) / Float(target.height)
        uniforms.viewMatrix.columns.0.x = max(1, 1 / aspect)
        uniforms.viewMatrix.columns.1.y = max(1, aspect)
        uniforms.model1.matrix.columns.3 = SIMD4(-0.5, 0.7, 0, 1)
        uniforms.model1.timeScale = 45
        uniforms.model2.matrix.columns.3 = SIMD4(-0.95, -0.7, 0, 1)
        uniforms.model2.timeScale = 35

        let pass = MTLRenderPassDescriptor()
        pass.colorAttachments[0].texture = rotationTexture
        pass.colorAttachments[0].loadAction = .clear
        pass.colorAttachments[0].storeAction = .store
        pass.colorAttachments[0].clearColor = MTLClearColorMake(0.07, 0.07, 0.09, 1)
        guard let rotation = command.makeRenderCommandEncoder(descriptor: pass) else { throw BackdropError.encoder }
        rotation.label = "Artwork rotation (three instances)"
        rotation.setRenderPipelineState(rotationPipeline)
        rotation.setVertexBytes(&uniforms, length: MemoryLayout<BackdropUniforms>.stride, index: 1)
        rotation.setFragmentBytes(&uniforms, length: MemoryLayout<BackdropUniforms>.stride, index: 1)
        rotation.setFragmentTexture(current, index: 0)
        rotation.setFragmentTexture(previous ?? current, index: 1)
        rotation.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: 6, instanceCount: 3)
        rotation.endEncoding()

        blur.encode(commandBuffer: command, sourceTexture: rotationTexture, destinationTexture: blurredTexture)

        uniforms.saturation = 2
        pass.colorAttachments[0].texture = target
        guard let pinch = command.makeRenderCommandEncoder(descriptor: pass) else { throw BackdropError.encoder }
        pinch.label = "Artwork pinch mesh"
        pinch.setRenderPipelineState(pinchPipeline)
        pinch.setVertexBuffer(meshBuffer, offset: 0, index: 0)
        pinch.setVertexBytes(&uniforms, length: MemoryLayout<BackdropUniforms>.stride, index: 1)
        pinch.setFragmentBytes(&uniforms, length: MemoryLayout<BackdropUniforms>.stride, index: 1)
        pinch.setFragmentTexture(blurredTexture, index: 0)
        pinch.drawIndexedPrimitives(type: .triangle, indexCount: indexCount, indexType: .uint32,
                                    indexBuffer: indexBuffer, indexBufferOffset: 0)
        pinch.endEncoding()
    }

    private func resize(width: Int, height: Int) throws {
        // ponytail: cap intermediate long edge at 960px; lift for pixel-exact GPU comparisons.
        let scale = min(1.0, 960.0 / Double(max(width, height)))
        let w = max(1, Int(Double(width) * scale)), h = max(1, Int(Double(height) * scale))
        guard rotationTexture?.width != w || rotationTexture?.height != h else { return }
        let descriptor = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: .bgra8Unorm,
                                                                 width: w, height: h, mipmapped: false)
        descriptor.storageMode = .private
        descriptor.usage = [.renderTarget, .shaderRead, .shaderWrite]
        guard let rotation = device.makeTexture(descriptor: descriptor),
              let blurred = device.makeTexture(descriptor: descriptor) else { throw BackdropError.allocation }
        let sigma = Float(floor(hypot(Double(width), Double(height)) * 0.045394707) * scale)
        let filter = MPSImageGaussianBlur(device: device, sigma: max(sigma, 1))
        filter.edgeMode = .zero
        rotationTexture = rotation
        blurredTexture = blurred
        blur = filter
    }

    static func makeMesh() -> (vertices: [BackdropVertex], indices: [UInt32]) {
        // Control-pair 0 at Music arm64e 0x101885858. Five divisions, six points per axis.
        let from: [SIMD2<Float>] = [
            .init(0,0), .init(0.2,0), .init(0.4,0), .init(0.6,0), .init(0.8,0), .init(1,0),
            .init(0,0.2), .init(-0.0933,0.4), .init(0.4,0.2), .init(0.6,0.2), .init(0.3653,0.1335), .init(1,0.2),
            .init(0,0.4), .init(0.4232,0.359), .init(0.3429,0.5349), .init(0.6,0.4), .init(0.832,0.4148), .init(1,0.4),
            .init(0,0.6), .init(0.2,0.6), .init(0.2293,0.7775), .init(0.7829,0.5595), .init(0.6514,0.7302), .init(1,0.6),
            .init(0,0.8), .init(0.2,0.8), .init(0.28,0.9195), .init(0.4773,0.8), .init(0.8,0.8), .init(1,0.8),
            .init(0,1), .init(0.6514,1.1073), .init(0.4,1), .init(1,1.0317), .init(1,1.1302), .init(1,1)
        ]
        var to = from
        to[10] = SIMD2(0.8587, 0.2234)
        to[13] = SIMD2(0.4526, 0.6053)
        var points = from.indices.map { i in
            SIMD8<Float>(Float(i % 6) / 5, Float(i / 6) / 5,
                         from[i].x, from[i].y, to[i].x, to[i].y, 0, 0)
        }
        var faces: [[Int]] = []
        for y in 0..<5 { for x in 0..<5 {
            let a = y * 6 + x
            faces.append([a, a + 1, a + 7, a + 6])
        } }
        // ponytail: public Catmull–Clark approximation of private CAMeshTransform subdividedMesh:3.
        // One recovered control pair; add the other four only after visual comparison.
        for _ in 0..<3 {
            struct Edge { var a: Int; var b: Int; var faces: [Int] }
            var edges: [Edge] = []
            var edgeIDs: [UInt64: Int] = [:]
            var faceEdges = Array(repeating: [Int](), count: faces.count)
            var vertexFaces = Array(repeating: [Int](), count: points.count)
            var vertexEdges = Array(repeating: [Int](), count: points.count)
            let centers = faces.map { face in face.reduce(SIMD8<Float>.zero) { $0 + points[$1] } / 4 }
            for (fi, face) in faces.enumerated() {
                for j in 0..<4 {
                    let a = face[j], b = face[(j + 1) % 4]
                    vertexFaces[a].append(fi)
                    let key = UInt64(min(a, b)) << 32 | UInt64(max(a, b))
                    let ei: Int
                    if let existing = edgeIDs[key] {
                        ei = existing
                        edges[ei].faces.append(fi)
                    } else {
                        ei = edges.count
                        edgeIDs[key] = ei
                        edges.append(Edge(a: a, b: b, faces: [fi]))
                        vertexEdges[a].append(ei)
                        vertexEdges[b].append(ei)
                    }
                    faceEdges[fi].append(ei)
                }
            }
            var refined = points
            for i in points.indices {
                let boundary = vertexEdges[i].filter { edges[$0].faces.count == 1 }
                if boundary.count == 2 && vertexFaces[i].count > 1 {
                    let neighbors = boundary.map { edges[$0].a == i ? edges[$0].b : edges[$0].a }
                    refined[i] = points[i] * 0.75 + (points[neighbors[0]] + points[neighbors[1]]) * 0.125
                } else if boundary.isEmpty {
                    let n = Float(vertexEdges[i].count)
                    let f = vertexFaces[i].reduce(SIMD8<Float>.zero) { $0 + centers[$1] } / n
                    let r = vertexEdges[i].reduce(SIMD8<Float>.zero) {
                        $0 + (points[edges[$1].a] + points[edges[$1].b]) * 0.5
                    } / n
                    refined[i] = (f + 2 * r + (n - 3) * points[i]) / n
                }
                refined[i][0] = points[i][0]
                refined[i][1] = points[i][1]
            }
            let edgeBase = refined.count
            for edge in edges {
                let midpoint = (points[edge.a] + points[edge.b]) * 0.5
                var p = midpoint
                if edge.faces.count == 2 {
                    p = (points[edge.a] + points[edge.b] + centers[edge.faces[0]] + centers[edge.faces[1]]) * 0.25
                }
                p[0] = midpoint[0]; p[1] = midpoint[1]
                refined.append(p)
            }
            let faceBase = refined.count
            refined += centers
            var subdivided: [[Int]] = []
            for (fi, face) in faces.enumerated() { for j in 0..<4 {
                subdivided.append([face[j], edgeBase + faceEdges[fi][j], faceBase + fi,
                                   edgeBase + faceEdges[fi][(j + 3) % 4]])
            } }
            points = refined
            faces = subdivided
        }
        let vertices = points.map { p in
            BackdropVertex(position: SIMD4(p[0], p[1], 0, 1),
                           from: SIMD4(p[2] * 2 - 1, p[3] * 2 - 1, 0, 1),
                           to: SIMD4(p[4] * 2 - 1, p[5] * 2 - 1, 0, 1), uv: SIMD2(p[0], p[1]))
        }
        let indices = faces.flatMap { [$0[0], $0[2], $0[3], $0[0], $0[1], $0[2]].map(UInt32.init) }
        return (vertices, indices)
    }

    static let shaderSource = """
    #include <metal_stdlib>
    using namespace metal;
    struct Model { float4x4 mtx; float timeScale; };
    struct Uniforms {
        float4x4 viewMatrix;
        float time, textureTransitionMix, meshWarpTimeScale, saturation;
        float whiteScrimAlpha, blackScrimAlpha, factorForDarkMode, factorForLightMode;
        float floor, ceiling, padding[3];
        Model models[3];
    };
    static_assert(sizeof(Uniforms) == 368, "Uniform layout");
    struct RotationOut { float4 position [[position]]; float2 uv; };
    float4x4 rotation(float angle) {
        float c = cos(angle), s = sin(angle);
        return float4x4(float4(c,-s,0,0), float4(s,c,0,0), float4(0,0,1,0), float4(0,0,0,1));
    }
    vertex RotationOut rotation_vertex(uint id [[vertex_id]], uint instance [[instance_id]],
                                       constant Uniforms& u [[buffer(1)]]) {
        constexpr float2 positions[] = { {-1,-1}, {-1,1}, {1,1}, {-1,-1}, {1,1}, {1,-1} };
        constexpr float2 uv[] = { {0,1}, {0,0}, {1,0}, {0,1}, {1,0}, {1,1} };
        float4x4 r = rotation(u.time * 2 * M_PI_F / u.models[instance].timeScale);
        return { u.viewMatrix * r * u.models[instance].mtx * r * float4(positions[id],0,1), uv[id] };
    }
    half3 saturateColor(half3 rgb, float amount) {
        half luma = dot(rgb, half3(0.3h, 0.59h, 0.11h));
        return luma + (rgb - luma) * half(amount);
    }
    fragment half4 rotation_fragment(RotationOut in [[stage_in]], constant Uniforms& u [[buffer(1)]],
                                      texture2d<half> current [[texture(0)]], texture2d<half> previous [[texture(1)]]) {
        constexpr sampler s(coord::normalized, address::clamp_to_edge, filter::linear);
        half3 rgb = mix(current.sample(s, in.uv).rgb, previous.sample(s, in.uv).rgb, half(u.textureTransitionMix));
        return half4(saturateColor(rgb, u.saturation), 1);
    }
    struct PinchIn {
        float4 position [[attribute(0)]];
        float4 fromPosition [[attribute(1)]];
        float4 toPosition [[attribute(2)]];
        float2 uv [[attribute(3)]];
    };
    struct PinchOut { float4 position [[position]]; float2 uv; };
    vertex PinchOut pinch_vertex(PinchIn in [[stage_in]], constant Uniforms& u [[buffer(1)]]) {
        float phase = acos(sin(u.time * M_PI_F / u.meshWarpTimeScale)) / M_PI_F;
        float blend = phase * phase * (3 - 2 * phase);
        return { float4(mix(in.fromPosition.xy, in.toPosition.xy, blend), 0, 1), (in.uv - 0.5) * 0.8 + 0.5 };
    }
    fragment half4 pinch_fragment(PinchOut in [[stage_in]], constant Uniforms& u [[buffer(1)]],
                                  texture2d<half> blurred [[texture(0)]]) {
        constexpr sampler s(coord::normalized, address::clamp_to_edge, filter::linear);
        half3 rgb = min(saturateColor(blurred.sample(s, in.uv).rgb, u.saturation), half3(0.995h));
        if (u.factorForDarkMode > 0) {
            rgb = mix(rgb, half3(0), half(u.blackScrimAlpha)) - half(u.factorForDarkMode * 0.1);
        } else if (u.factorForLightMode > 0) {
            rgb = min(mix(rgb, half3(1), half(u.whiteScrimAlpha)) + half(u.factorForLightMode * 0.05), half3(1));
        }
        return half4(clamp(rgb, half3(u.floor), half3(u.ceiling)), 1);
    }
    """
}
