import SwiftUI

/// Transient capsule message at the bottom of the window (`ToastState`).
struct ToastView: View {
    let message: String

    var body: some View {
        VStack {
            Spacer()
            Text(message)
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.white)
                .padding(.horizontal, Layout.Toast.horizontalPadding)
                .padding(.vertical, Layout.Toast.verticalPadding)
                .background(.ultraThinMaterial)
                .clipShape(Capsule())
                .shadow(color: .black.opacity(0.3), radius: 8, x: 0, y: 4)
                .padding(.bottom, Layout.Toast.bottomOffset)
                .transition(.move(edge: .bottom).combined(with: .opacity))
        }
        .zIndex(20)
    }
}
