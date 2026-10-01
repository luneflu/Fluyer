import SwiftUI
import FluyerCore

extension Color {
    init(rgb: ColorRgb, opacity: Double = 1.0) {
        self.init(
            .sRGB,
            red: Double(rgb.r) / 255.0,
            green: Double(rgb.g) / 255.0,
            blue: Double(rgb.b) / 255.0,
            opacity: opacity
        )
    }

    static let defaultBackground = Color(red: 20/255, green: 20/255, blue: 28/255)
    static let cardBackground = Color.white.opacity(0.06)
    static let cardBorder = Color.white.opacity(0.12)
}
