import SwiftUI

/// Toolbar sort control (legacy filter-bar sort toggle). Keys follow the current
/// mode; the direction is shared.
struct SortMenu: View {
    @Bindable var selection: LibraryFilterState

    var body: some View {
        Menu {
            if selection.mode == .albums {
                Picker("Sort By", selection: $selection.albumSort) {
                    ForEach(AlbumSort.allCases) { Text($0.rawValue).tag($0) }
                }
            } else {
                Picker("Sort By", selection: $selection.trackSort) {
                    ForEach(TrackSort.allCases) { Text($0.rawValue).tag($0) }
                }
            }
            Divider()
            Picker("Order", selection: $selection.sortAscending) {
                Text("Ascending").tag(true)
                Text("Descending").tag(false)
            }
        } label: {
            Label("Sort", systemImage: "arrow.up.arrow.down")
        }
        .pickerStyle(.inline)
        .labelStyle(.iconOnly)
        .menuIndicator(.hidden)
        .fixedSize()
        .help("Sort")
    }
}
