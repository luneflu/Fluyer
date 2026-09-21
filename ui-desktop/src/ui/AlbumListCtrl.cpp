#include "AlbumListCtrl.h"
#include <wx/dcbuffer.h>
#include <wx/settings.h>
#include <algorithm>

struct ResponsiveRule {
    int minWidth;
    double minDpr;
    double widthRatio;
};

static constexpr ResponsiveRule RESPONSIVE_RULES[] = {
    { 1536, 1.01, 0.142857 }, // hdpi 2xl → 14.2857%
    { 1280, 1.01, 0.16667 },  // xl-hdpi → 16.6667%
    { 1024, 1.01, 0.2 },      // lg-hdpi → 20%
    { 768,  1.01, 0.25 },     // md-hdpi → 25%
    { 640,  1.01, 0.33334 },  // sm-hdpi → 33.3334%

    { 1536, 0.0,  0.125 },    // 2xl → 12.5%
    { 1440, 0.0,  0.142857 }, // 1440 → 14.2857%
    { 1280, 0.0,  0.16667 },  // xl → 16.6667%
    { 1024, 0.0,  0.2 },      // lg → 20%
    { 768,  0.0,  0.25 },     // md → 25%
    { 640,  0.0,  0.33334 }   // sm → 33.3334%
};

wxBEGIN_EVENT_TABLE(AlbumListCtrl, wxScrolledWindow)
    EVT_PAINT(AlbumListCtrl::OnPaint)
    EVT_SIZE(AlbumListCtrl::OnSize)
    EVT_LEFT_DOWN(AlbumListCtrl::OnLeftDown)
    EVT_MOUSEWHEEL(AlbumListCtrl::OnMouseWheel)
wxEND_EVENT_TABLE()

AlbumListCtrl::AlbumListCtrl(wxWindow* parent, LibraryService* libraryService, ImageService* imageService, wxWindowID id)
    : wxScrolledWindow(parent, id, wxDefaultPosition, wxDefaultSize, wxNO_BORDER | wxFULL_REPAINT_ON_RESIZE),
      m_libraryService(libraryService),
      m_imageService(imageService) {
    ShowScrollbars(wxSHOW_SB_NEVER, wxSHOW_SB_NEVER);
    SetBackgroundStyle(wxBG_STYLE_PAINT);
}

int AlbumListCtrl::GetItemWidth() const {
    int width = GetClientSize().GetWidth();
    if (width <= 0 && GetParent()) {
        width = GetParent()->GetClientSize().GetWidth();
    }
    double dpr = GetContentScaleFactor();

    for (const auto& rule : RESPONSIVE_RULES) {
        if (width >= rule.minWidth && dpr >= rule.minDpr) {
            return std::max(1, static_cast<int>(rule.widthRatio * width));
        }
    }
    return std::max(1, static_cast<int>(0.5 * width));
}

int AlbumListCtrl::GetItemHeight() const {
    if (m_albumCount == 0) return 0;
    int itemWidth = GetItemWidth();
    int padding = 6;
    int coverSize = std::max(16, itemWidth - padding * 2);
    int topMargin = 8;
    int titleSpacing = 6;
    int artistSpacing = 24;
    int bottomMargin = 8;
    // Total: topMargin + coverSize + artistSpacing + font height (~13px) + bottomMargin
    return topMargin + coverSize + artistSpacing + 13 + bottomMargin;
}

wxSize AlbumListCtrl::DoGetBestClientSize() const {
    return wxSize(wxDefaultCoord, GetItemHeight());
}

void AlbumListCtrl::RefreshData() {
    m_scrollOffsetX = 0;
    m_albumCount = m_libraryService ? m_libraryService->GetAlbumCount() : 0;
    SetMinSize(wxSize(-1, GetItemHeight()));
    if (GetParent()) {
        GetParent()->Layout();
    }
    Refresh();
}

void AlbumListCtrl::OnCoverLoaded(uintptr_t index) {
    if (m_imageService) {
        m_imageService->InvalidateAlbumCover(index);
    }
    Refresh();
}

void AlbumListCtrl::OnSize(wxSizeEvent& evt) {
    int itemWidth = GetItemWidth();
    int itemHeight = GetItemHeight();
    SetMinSize(wxSize(-1, itemHeight));
    int totalWidth = static_cast<int>(m_albumCount) * itemWidth;
    int maxScroll = std::max(0, totalWidth - GetClientSize().GetWidth());
    m_scrollOffsetX = std::clamp(m_scrollOffsetX, 0, maxScroll);
    Refresh();
    evt.Skip();
}

void AlbumListCtrl::OnMouseWheel(wxMouseEvent& evt) {
    int itemWidth = GetItemWidth();
    int totalWidth = static_cast<int>(m_albumCount) * itemWidth;
    int maxScroll = std::max(0, totalWidth - GetClientSize().GetWidth());
    int delta = evt.GetWheelRotation();
    m_scrollOffsetX = std::clamp(m_scrollOffsetX - delta, 0, maxScroll);
    Refresh();
}

void AlbumListCtrl::OnLeftDown(wxMouseEvent& evt) {
    int itemWidth = GetItemWidth();
    if (itemWidth > 0) {
        int x = evt.GetX() + m_scrollOffsetX;
        int clickedIdx = x / itemWidth;
        if (clickedIdx >= 0 && clickedIdx < static_cast<int>(m_albumCount)) {
            // Future: Filter or play album tracks
        }
    }
    evt.Skip();
}

void AlbumListCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);

    dc.SetBackground(wxBrush(GetBackgroundColour()));
    dc.Clear();

    if (m_albumCount == 0 || !m_libraryService) return;

    int itemWidth = GetItemWidth();
    if (itemWidth <= 0) return;

    int viewW = GetClientSize().GetWidth();
    int startPixelX = m_scrollOffsetX;
    int endPixelX = startPixelX + viewW;

    int startIdx = std::max(0, startPixelX / itemWidth);
    int endIdx = std::min(static_cast<int>(m_albumCount) - 1, endPixelX / itemWidth + 1);

    double scaleFactor = GetContentScaleFactor();
    wxFont titleFont = wxFontInfo(wxSize(0, 15)).Weight(wxFONTWEIGHT_MEDIUM).Family(wxFONTFAMILY_DEFAULT);
    wxFont artistFont = wxFontInfo(wxSize(0, 13)).Family(wxFONTFAMILY_DEFAULT);

    int padding = 6;
    int coverSize = itemWidth - padding * 2;
    
    // Track visible albums for cache management
    if (m_imageService) {
        m_imageService->BeginVisibilityUpdate();
    }

    for (int i = startIdx; i <= endIdx; ++i) {
        if (m_imageService) {
            m_imageService->MarkAlbumVisible(i);
        }
        
        int itemX = i * itemWidth - m_scrollOffsetX;
        int coverX = itemX + padding;
        int itemY = 8;

        Album album = m_libraryService->GetAlbum(i);
        wxBitmap cover;
        if (m_imageService) {
            cover = m_imageService->GetAlbumImage(i, coverSize, scaleFactor);
        }

        if (cover.IsOk()) {
            dc.DrawBitmap(cover, coverX, itemY, false);
        } else {
            dc.SetBrush(wxBrush(wxSystemSettings::GetColour(wxSYS_COLOUR_BTNFACE)));
            dc.SetPen(wxPen(wxSystemSettings::GetColour(wxSYS_COLOUR_BTNSHADOW)));
            dc.DrawRoundedRectangle(coverX, itemY, coverSize, coverSize, 8.0);
        }

        // Title
        dc.SetTextForeground(GetForegroundColour());
        dc.SetFont(titleFont);
        wxString albumName = wxString::FromUTF8(album.name);
        wxString truncatedTitle = albumName;
        if (dc.GetTextExtent(truncatedTitle).GetWidth() > coverSize) {
            while (!truncatedTitle.empty() && dc.GetTextExtent(truncatedTitle + "...").GetWidth() > coverSize) {
                truncatedTitle.RemoveLast();
            }
            truncatedTitle += "...";
        }
        dc.DrawText(truncatedTitle, coverX, itemY + coverSize + 6);

        // Artist
        dc.SetTextForeground(wxSystemSettings::GetColour(wxSYS_COLOUR_GRAYTEXT));
        dc.SetFont(artistFont);
        wxString artistName = wxString::FromUTF8(album.artist);
        wxString truncatedArtist = artistName;
        if (dc.GetTextExtent(truncatedArtist).GetWidth() > coverSize) {
            while (!truncatedArtist.empty() && dc.GetTextExtent(truncatedArtist + "...").GetWidth() > coverSize) {
                truncatedArtist.RemoveLast();
            }
            truncatedArtist += "...";
        }
        dc.DrawText(truncatedArtist, coverX, itemY + coverSize + 24);
    }
    
    // Cleanup images no longer visible
    if (m_imageService) {
        m_imageService->EndVisibilityUpdate();
    }
}
