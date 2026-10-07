import SwiftUI
import AppKit

/// Native `NSSearchField` for the toolbar: system border, magnifier and clear
/// button, Esc clears, plus a white focus ring.
///
/// ponytail: `.searchable(placement: .toolbar)` pins its field to the trailing edge,
/// so nothing can sit to its right; wrapping the AppKit field lets the queue toggle
/// share the trailing group.
struct ToolbarSearchField: NSViewRepresentable {
    @Binding var text: String
    let prompt: String

    func makeNSView(context: Context) -> NSSearchField {
        let field = WhiteFocusSearchField()
        field.placeholderString = prompt
        field.sendsSearchStringImmediately = true
        field.delegate = context.coordinator
        return field
    }

    func updateNSView(_ field: NSSearchField, context: Context) {
        context.coordinator.text = $text
        if field.stringValue != text { field.stringValue = text }
    }

    func makeCoordinator() -> Coordinator { Coordinator(text: $text) }

    final class Coordinator: NSObject, NSSearchFieldDelegate {
        var text: Binding<String>

        init(text: Binding<String>) { self.text = text }

        func controlTextDidChange(_ note: Notification) {
            guard let field = note.object as? NSSearchField else { return }
            text.wrappedValue = field.stringValue
        }
    }
}

/// `NSSearchField` whose focus ring is white instead of the system accent colour.
///
/// ponytail: AppKit always paints the system ring in the user's accent colour and
/// `.tint` can't reach it, so the system ring is off and this view draws its own
/// while the field editor is active. The rounded rect follows the bezel measured
/// from an offscreen render (1pt inset, 6pt radius); the 3pt stroke stays inside
/// the view's bounds, so nothing is clipped.
private final class WhiteFocusSearchField: NSSearchField {
    private let ringWidth: CGFloat = 1
    private let ringRadius: CGFloat = 6

    override init(frame: NSRect) {
        super.init(frame: frame)
        focusRingType = .none
    }

    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

    private var isEditing: Bool {
        guard let editor = window?.firstResponder as? NSText else { return false }
        return editor.delegate === self
    }

    override func becomeFirstResponder() -> Bool {
        let accepted = super.becomeFirstResponder()
        needsDisplay = true
        return accepted
    }

    override func textDidEndEditing(_ notification: Notification) {
        super.textDidEndEditing(notification)
        needsDisplay = true
    }

    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)
        guard isEditing else { return }
        let rect = bounds.insetBy(dx: 1, dy: 1)
        let path = NSBezierPath(roundedRect: rect, xRadius: ringRadius, yRadius: ringRadius)
        path.lineWidth = ringWidth
        NSColor.white.withAlphaComponent(0.85).setStroke()
        path.stroke()
    }
}
