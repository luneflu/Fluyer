import CoreGraphics

/// Every padding, spacing, size and corner radius in the app, grouped by the UI
/// part it belongs to. Search the feature name ("Queue", "PlayerBar") to land here.
///
/// Names say what a value is for, not what it is, so two parts that happen to share
/// a number stay independent. Font sizes live with their views.
enum Layout {
    /// Window shell (`HomeView`).
    enum Window {
        static let minWidth: CGFloat = 920
        static let minHeight: CGFloat = 680
        static let toolbarSpacing: CGFloat = 8
        static let searchFieldWidth: CGFloat = 260
        static let scanProgressSpacing: CGFloat = 6
    }

    /// Shared row look for song grid rows and queue rows, so covers share a cache key.
    enum TrackRow {
        static let coverSide: CGFloat = 44
        /// Requested cover pixels (2x for Retina).
        static let coverPixels = 88
        static let coverCornerRadius: CGFloat = 4
        static let coverToText: CGFloat = 10
        static let titleToSubtitle: CGFloat = 2
        static let iconToTitle: CGFloat = 6
        static let minTextToTrailing: CGFloat = 4
        static let verticalPadding: CGFloat = 6
    }

    /// Song grid (`MusicGridView`).
    enum MusicGrid {
        static let minColumnWidth: CGFloat = 280
        static let columnSpacing: CGFloat = 12
        static let rowSpacing: CGFloat = 8
        static let horizontalPadding: CGFloat = 16
        static let verticalPadding: CGFloat = 8
        static let emptyStateSpacing: CGFloat = 12
        static let emptyStateButtonTop: CGFloat = 4
    }

    /// Album strip, album grid and album cards. Card sizing math is in `AlbumMetrics`.
    enum Albums {
        static let cardGap: CGFloat = 12
        static let edge: CGFloat = 16
        static let cardLabelHeight: CGFloat = 52
        static let gridVerticalPadding: CGFloat = 8
        static let coverToLabels: CGFloat = 5
        static let nameToArtist: CGFloat = 2
        static let coverCornerRadius: CGFloat = 4
    }

    /// Header shown above the song grid while one album is open.
    enum AlbumHeader {
        static let spacing: CGFloat = 12
        static let buttonSpacing: CGFloat = 8
        static let horizontalPadding: CGFloat = 16
        static let height: CGFloat = 46
        static let cornerRadius: CGFloat = 8
        static let outerHorizontalPadding: CGFloat = 16
        static let outerTopPadding: CGFloat = 4
        static let buttonSide: CGFloat = 28
    }

    /// Queue panel ("Now Playing").
    enum Queue {
        /// Gap between the panel and the window's right and bottom edges.
        static let outerInset: CGFloat = 12
        /// Gap between the panel and the toolbar.
        static let outerTop: CGFloat = 6
        static let panelPadding: CGFloat = 12
        static let headerPadding: CGFloat = 12
        static let headerSpacing: CGFloat = 12
        static let clearButtonSide: CGFloat = 28
        static let cornerRadius: CGFloat = 4
        static let rowHorizontalPadding: CGFloat = 12
        static let removeButtonSide: CGFloat = 24
    }

    /// Bottom player bar on the home screen.
    enum PlayerBar {
        static let seekToControls: CGFloat = 6
        static let seekThickness: CGFloat = 3
        static let seekThicknessHovered: CGFloat = 5
        static let seekHitHeight: CGFloat = 6
        static let seekHorizontalPadding: CGFloat = 16
        static let sideColumnWidth: CGFloat = 190
        static let minGapToCenter: CGFloat = 16
        static let controlsHorizontalPadding: CGFloat = 18
        static let controlsHeight: CGFloat = 52
        static let horizontalPadding: CGFloat = 16
        static let bottomPadding: CGFloat = 10
        static let transportSpacing: CGFloat = 12
        static let coverSide: CGFloat = 40
        static let coverToText: CGFloat = 10
        static let titleToArtist: CGFloat = 2
        static let secondarySpacing: CGFloat = 10
        static let volumeSliderWidth: CGFloat = 80
        static let volumeSliderHeight: CGFloat = 24
        static let iconButtonSide: CGFloat = 24
    }

    /// Full-screen play view.
    enum Play {
        static let topPadding: CGFloat = 24
        static let trailingPadding: CGFloat = 20
        /// Column width as a fraction of the window, with and without lyrics.
        static let coverColumnRatio: CGFloat = 0.40
        static let centeredColumnRatio: CGFloat = 0.50
        static let lyricsColumnRatio: CGFloat = 0.55
        /// Gutter between the cover column and the lyrics.
        static let columnGutter: CGFloat = 40
        static let minCoverSide: CGFloat = 200
        static let maxCoverSide: CGFloat = 360
        static let coverToControls: CGFloat = 16
        static let backButtonSide: CGFloat = 28
        static let backButtonLeading: CGFloat = 12
        static let backButtonTop: CGFloat = 24
    }

    /// Control card under the cover on the play view.
    enum PlayControls {
        static let infoTopPadding: CGFloat = 4
        static let infoBottomPadding: CGFloat = 12
        static let seekHeight: CGFloat = 8
        static let transportTopPadding: CGFloat = 4
        static let infoSpacing: CGFloat = 10
        static let timeLabelWidth: CGFloat = 48
        static let transportSpacing: CGFloat = 8
        static let volumeSpacing: CGFloat = 12
        static let volumeIconWidth: CGFloat = 20
        static let volumeSliderHeight: CGFloat = 14
        /// Hit area around each icon: icon size + this.
        static let iconButtonPadding: CGFloat = 18
    }

    /// Lyrics column on the play view.
    enum Lyrics {
        static let lineSpacing: CGFloat = 28
        /// Lets the first and last line scroll to the centre.
        static let verticalInset: CGFloat = 260
        static let horizontalPadding: CGFloat = 28
        static let lineVerticalPadding: CGFloat = 20
        static let noteWidth: CGFloat = 32
        static let noteWidthActive: CGFloat = 38
    }

    /// Bottom toast message.
    enum Toast {
        static let horizontalPadding: CGFloat = 16
        static let verticalPadding: CGFloat = 10
        static let bottomOffset: CGFloat = 80
    }

    /// Settings window.
    enum Settings {
        static let width: CGFloat = 520
        static let minHeight: CGFloat = 380
    }
}
