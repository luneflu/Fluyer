// swift-tools-version: 5.9
import PackageDescription
import Foundation

let packageRoot = URL(fileURLWithPath: #file).deletingLastPathComponent().path
let repoRoot = URL(fileURLWithPath: packageRoot).deletingLastPathComponent().deletingLastPathComponent().path

/// ponytail: the app links BASS and the Rust core statically-by-path, and the test
/// bundle links the app's object files, so it needs the exact same search paths,
/// libraries and rpaths or it fails to load with undefined symbols at startup.
let nativeLinkerSettings: [LinkerSetting] = [
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
            linkerSettings: nativeLinkerSettings
        ),
        .testTarget(
            name: "FluyerAppTests",
            dependencies: [
                "FluyerApp",
                .product(name: "FluyerCore", package: "swift")
            ],
            path: "Tests/FluyerAppTests",
            linkerSettings: nativeLinkerSettings
        )
    ]
)
