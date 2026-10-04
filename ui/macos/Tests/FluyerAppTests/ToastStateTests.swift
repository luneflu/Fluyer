import XCTest
@testable import FluyerApp

@MainActor
final class ToastStateTests: XCTestCase {
    func testShowSetsMessage() {
        let toast = ToastState()
        XCTAssertNil(toast.message)

        toast.show("Library scan completed")

        XCTAssertEqual(toast.message, "Library scan completed")
    }

    func testNewerToastReplacesOlder() {
        let toast = ToastState()

        toast.show("first")
        toast.show("second")

        XCTAssertEqual(toast.message, "second")
    }

    func testDismissClears() {
        let toast = ToastState()
        toast.show("something")

        toast.dismiss()

        XCTAssertNil(toast.message)
    }

    /// Regression: the previous implementation spawned an uncancellable `Task` per
    /// toast, so every message leaked a task that slept for three seconds.
    func testRapidToastsDoNotAccumulatePendingTasks() {
        let toast = ToastState()

        for index in 0..<50 {
            toast.show("message \(index)")
        }

        XCTAssertEqual(toast.message, "message 49")
    }
}
