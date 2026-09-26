// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FluyerCore",
    platforms: [
        .macOS(.v13),
        .iOS(.v16)
    ],
    products: [
        .library(
            name: "FluyerCore",
            targets: ["FluyerCore"]
        ),
    ],
    targets: [
        .target(
            name: "fluyer_coreFFI",
            path: "Sources/fluyer_coreFFI",
            publicHeadersPath: "."
        ),
        .target(
            name: "FluyerCore",
            dependencies: ["fluyer_coreFFI"],
            path: "Sources/FluyerCore"
        ),
    ]
)
