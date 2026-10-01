// swift-tools-version: 5.9
import PackageDescription
import Foundation

let packageRoot = URL(fileURLWithPath: #file).deletingLastPathComponent().path
let repoRoot = URL(fileURLWithPath: packageRoot).deletingLastPathComponent().deletingLastPathComponent().path

let package = Package(
    name: "FluyerSwiftUI",
    platforms: [
        .macOS(.v14)
    ],
    dependencies: [
        .package(path: "../../bindings/swift")
    ],
    targets: [
        .executableTarget(
            name: "FluyerApp",
            dependencies: [
                .product(name: "FluyerCore", package: "swift")
            ],
            path: "Sources",
            linkerSettings: [
                .unsafeFlags([
                    "-L\(repoRoot)/target/debug",
                    "-L\(repoRoot)/libs/macos",
                    "-lfluyer_core",
                    "-lbass",
                    "-lbassmix",
                    "-Xlinker", "-rpath", "-Xlinker", "\(repoRoot)/target/debug",
                    "-Xlinker", "-rpath", "-Xlinker", "\(repoRoot)/libs/macos",
                    "-Xlinker", "-rpath", "-Xlinker", "@executable_path/../Frameworks",
                    "-Xlinker", "-rpath", "-Xlinker", "@executable_path"
                ])
            ]
        )
    ]
)
