#include "AlbumListCtrl.h"
#include <wx/dcbuffer.h>
#include <wx/settings.h>
#include <algorithm>

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

void AlbumListCtrl::RefreshData() {
    m_scrollOffsetX = 0;
    m_albumCount = m_libraryService ? m_libraryService->GetAlbumCount() : 0;
    Refresh();
}

void AlbumListCtrl::OnCoverLoaded(uintptr_t index) {
    if (m_imageService) {
        m_imageService->InvalidateAlbumCover(index);
    }
    Refresh();
}

void AlbumListCtrl::OnSize(wxSizeEvent& evt) {
    Refresh();
    evt.Skip();
}

void AlbumListCtrl::OnMouseWheel(wxMouseEvent& evt) {
    int maxScroll = std::max(0, static_cast<int>(m_albumCount) * (ITEM_WIDTH + ITEM_SPACING) - GetClientSize().GetWidth() + ITEM_SPACING * 2);
    int delta = evt.GetWheelRotation();
    m_scrollOffsetX = std::clamp(m_scrollOffsetX - delta, 0, maxScroll);
    Refresh();
}

void AlbumListCtrl::OnLeftDown(wxMouseEvent& evt) {
    int x = evt.GetX() + m_scrollOffsetX;
    int clickedIdx = (x - ITEM_SPACING) / (ITEM_WIDTH + ITEM_SPACING);
    if (clickedIdx >= 0 && clickedIdx < static_cast<int>(m_albumCount)) {
        // Future: Filter or play album tracks
    }
    evt.Skip();
}

void AlbumListCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);

    dc.SetBackground(wxBrush(GetBackgroundColour()));
    dc.Clear();

    if (m_albumCount == 0 || !m_libraryService) return;

    int viewW = GetClientSize().GetWidth();
    int startPixelX = m_scrollOffsetX;
    int endPixelX = startPixelX + viewW;

    int startIdx = std::max(0, (startPixelX - ITEM_SPACING) / (ITEM_WIDTH + ITEM_SPACING));
    int endIdx = std::min(static_cast<int>(m_albumCount) - 1, (endPixelX - ITEM_SPACING) / (ITEM_WIDTH + ITEM_SPACING) + 1);

    double scaleFactor = GetContentScaleFactor();
    wxFont titleFont = wxFontInfo(wxSize(0, 13)).Bold().Family(wxFONTFAMILY_DEFAULT);
    wxFont artistFont = wxFontInfo(wxSize(0, 11)).Family(wxFONTFAMILY_DEFAULT);

    for (int i = startIdx; i <= endIdx; ++i) {
        int itemX = ITEM_SPACING + i * (ITEM_WIDTH + ITEM_SPACING) - m_scrollOffsetX;
        int itemY = 8;

        Album album = m_libraryService->GetAlbum(i);
        wxBitmap cover;
        if (m_imageService) {
            cover = m_imageService->GetAlbumImage(i, COVER_SIZE, scaleFactor);
        }

        if (cover.IsOk()) {
            dc.DrawBitmap(cover, itemX, itemY, false);
        } else {
            dc.SetBrush(wxBrush(wxSystemSettings::GetColour(wxSYS_COLOUR_BTNFACE)));
            dc.SetPen(wxPen(wxSystemSettings::GetColour(wxSYS_COLOUR_BTNSHADOW)));
            dc.DrawRoundedRectangle(itemX, itemY, COVER_SIZE, COVER_SIZE, 8.0);
        }

        // Title
        dc.SetTextForeground(GetForegroundColour());
        dc.SetFont(titleFont);
        wxString albumName = wxString::FromUTF8(album.name);
        wxString truncatedTitle = dc.GetTextExtent(albumName).GetWidth() > COVER_SIZE ? albumName.substr(0, 16) + "..." : albumName;
        dc.DrawText(truncatedTitle, itemX, itemY + COVER_SIZE + 6);

        // Artist
        dc.SetTextForeground(wxSystemSettings::GetColour(wxSYS_COLOUR_GRAYTEXT));
        dc.SetFont(artistFont);
        wxString artistName = wxString::FromUTF8(album.artist);
        wxString truncatedArtist = dc.GetTextExtent(artistName).GetWidth() > COVER_SIZE ? artistName.substr(0, 18) + "..." : artistName;
        dc.DrawText(truncatedArtist, itemX, itemY + COVER_SIZE + 24);
    }
}
