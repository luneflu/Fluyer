#include "MusicListCtrl.h"
#include "common/Log.h"
#include <wx/dcbuffer.h>
#include <algorithm>

struct ResponsiveRule {
    int minWidth;
    double minDpr;
    double widthRatio;
};

static constexpr ResponsiveRule RESPONSIVE_RULES[] = {
    { 1280, 2.01, 0.25 },    // 4 cols at 1280+ DPR>2
    { 1024, 2.01, 0.33333 }, // 3 cols
    { 768,  2.01, 0.5 },     // 2 cols
    { 1536, 1.01, 0.25 },    // hdpi 4 cols
    { 1280, 1.01, 0.33333 }, // 3 cols
    { 768,  1.01, 0.5 },     // 2 cols
    { 1536, 0.0,  0.25 },    // 4 cols
    { 1024, 0.0,  0.33333 }, // 3 cols
    { 768,  0.0,  0.5 }      // 2 cols
};

wxBEGIN_EVENT_TABLE(MusicListCtrl, wxScrolledWindow)
    EVT_PAINT(MusicListCtrl::OnPaint)
    EVT_SIZE(MusicListCtrl::OnSize)
    EVT_LEFT_DOWN(MusicListCtrl::OnLeftDown)
    EVT_MOUSEWHEEL(MusicListCtrl::OnMouseWheel)
wxEND_EVENT_TABLE()

MusicListCtrl::MusicListCtrl(wxWindow* parent, LibraryService* libraryService, ImageService* imageService, wxWindowID id)
    : wxScrolledWindow(parent, id, wxDefaultPosition, wxDefaultSize, wxNO_BORDER | wxFULL_REPAINT_ON_RESIZE),
      m_libraryService(libraryService),
      m_imageService(imageService) {
    ShowScrollbars(wxSHOW_SB_NEVER, wxSHOW_SB_NEVER);
    SetBackgroundStyle(wxBG_STYLE_PAINT);
}

void MusicListCtrl::SetAlbumFilter(int albumIndex) {
    m_albumFilter = albumIndex;
    RefreshData();
}

void MusicListCtrl::RefreshData() {
    m_scrollOffsetY = 0;
    if (m_albumFilter >= 0 && m_libraryService) {
        m_filteredAlbum = m_libraryService->GetAlbum(m_albumFilter);
        m_trackCount = m_filteredAlbum.tracks.size();
    } else {
        m_filteredAlbum = Album{};
        m_trackCount = m_libraryService ? m_libraryService->GetTrackCount() : 0;
    }
    Refresh();
}

void MusicListCtrl::OnCoverLoaded(uintptr_t index) {
    if (m_imageService) {
        m_imageService->InvalidateTrackCover(index);
    }
    Refresh();
}

void MusicListCtrl::OnSize(wxSizeEvent& evt) {
    Refresh();
    evt.Skip();
}

int MusicListCtrl::GetColumnCount() const {
    int width = GetClientSize().GetWidth();
    if (width <= 0) return 1;
    double dpr = GetContentScaleFactor();

    for (const auto& rule : RESPONSIVE_RULES) {
        if (width >= rule.minWidth && dpr >= rule.minDpr) {
            return std::max(1, static_cast<int>(1.0 / rule.widthRatio));
        }
    }
    return 1;
}

void MusicListCtrl::OnMouseWheel(wxMouseEvent& evt) {
    int colCount = GetColumnCount();
    int totalRows = (static_cast<int>(m_trackCount) + colCount - 1) / colCount;
    int maxScroll = std::max(0, totalRows * ITEM_HEIGHT - GetClientSize().GetHeight() + PADDING * 2);

    int delta = evt.GetWheelRotation();
    m_scrollOffsetY = std::clamp(m_scrollOffsetY - delta, 0, maxScroll);
    Refresh();
}

void MusicListCtrl::OnLeftDown(wxMouseEvent& evt) {
    int x = evt.GetX();
    int y = evt.GetY() + m_scrollOffsetY;
    int clientW = GetClientSize().GetWidth();
    int colCount = GetColumnCount();
    int colWidth = (clientW - PADDING * 2) / colCount;

    int clickedRow = (y - PADDING) / ITEM_HEIGHT;
    int clickedCol = (x - PADDING) / colWidth;

    if (clickedCol >= 0 && clickedCol < colCount && clickedRow >= 0) {
        size_t trackIdx = clickedRow * colCount + clickedCol;
        FluyerLog::Info("UI", "Clicked track index " + std::to_string(trackIdx));
        if (trackIdx < m_trackCount && m_libraryService) {
            if (m_albumFilter >= 0) {
                m_libraryService->PlayAlbumTrack(m_albumFilter, trackIdx);
            } else {
                m_libraryService->PlayTrack(trackIdx);
            }
        }
    }
    evt.Skip();
}

void MusicListCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);

    dc.SetBackground(wxBrush(GetBackgroundColour()));
    dc.Clear();

    if (m_trackCount == 0 || !m_libraryService) return;

    int viewW = GetClientSize().GetWidth();
    int viewH = GetClientSize().GetHeight();
    int startPixelY = m_scrollOffsetY;
    int endPixelY = startPixelY + viewH;

    int colCount = GetColumnCount();
    int colWidth = (viewW - PADDING * 2) / colCount;

    int startRow = std::max(0, (startPixelY - PADDING) / ITEM_HEIGHT);
    int endRow = (endPixelY - PADDING) / ITEM_HEIGHT + 1;

    double scaleFactor = GetContentScaleFactor();
    dc.SetFont(GetFont());
    dc.SetTextForeground(GetForegroundColour());
    
    // Track visible tracks for cache management
    if (m_imageService) {
        m_imageService->BeginVisibilityUpdate();
    }

    for (int row = startRow; row <= endRow; ++row) {
        for (int col = 0; col < colCount; ++col) {
            size_t idx = row * colCount + col;
            if (idx >= m_trackCount) break;

            Track track;
            wxBitmap thumb;
            if (m_albumFilter >= 0) {
                if (idx < m_filteredAlbum.tracks.size()) {
                    track = m_filteredAlbum.tracks[idx];
                }
                if (m_imageService) {
                    thumb = m_imageService->GetAlbumImage(m_albumFilter, THUMB_SIZE, scaleFactor);
                }
            } else {
                if (m_imageService) {
                    m_imageService->MarkTrackVisible(idx);
                }
                track = m_libraryService->GetTrack(idx);
                if (m_imageService) {
                    thumb = m_imageService->GetTrackImage(idx, THUMB_SIZE, scaleFactor);
                }
            }

            int cellX = PADDING + col * colWidth;
            int cellY = PADDING + row * ITEM_HEIGHT - m_scrollOffsetY;

            // Thumbnail
            int thumbX = cellX + 6;
            int thumbY = cellY + 4;
            if (thumb.IsOk()) {
                dc.DrawBitmap(thumb, thumbX, thumbY, false);
            }

            // Track metadata text
            int textX = thumbX + THUMB_SIZE + 8;
            int textMaxW = colWidth - THUMB_SIZE - 60;

            wxString tStr = wxString::FromUTF8(track.title);
            if (dc.GetTextExtent(tStr).GetWidth() > textMaxW && tStr.length() > 20) {
                tStr = tStr.substr(0, 18) + "...";
            }
            dc.DrawText(tStr, textX, cellY + 8);

            wxString subStr = wxString::FromUTF8(track.artist) + " • " + wxString::FromUTF8(track.album);
            if (dc.GetTextExtent(subStr).GetWidth() > textMaxW && subStr.length() > 24) {
                subStr = subStr.substr(0, 22) + "...";
            }
            dc.DrawText(subStr, textX, cellY + 28);

            // Duration on right
            dc.DrawText(wxString::FromUTF8(track.FormatDuration()), cellX + colWidth - 48, cellY + 18);
        }
    }
    
    // Cleanup images no longer visible
    if (m_imageService) {
        m_imageService->EndVisibilityUpdate();
    }
}
