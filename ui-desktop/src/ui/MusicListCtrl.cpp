#include "MusicListCtrl.h"
#include "common/Log.h"
#include <wx/dcbuffer.h>
#include <algorithm>

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
    SetBackgroundColour(wxColour(14, 14, 14));
}

void MusicListCtrl::RefreshData() {
    m_scrollOffsetY = 0;
    m_trackCount = m_libraryService ? m_libraryService->GetTrackCount() : 0;
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

void MusicListCtrl::OnMouseWheel(wxMouseEvent& evt) {
    int clientW = GetClientSize().GetWidth();
    int colCount = std::max(1, (clientW - PADDING * 2) / MIN_COL_WIDTH);
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
    int colCount = std::max(1, (clientW - PADDING * 2) / MIN_COL_WIDTH);
    int colWidth = (clientW - PADDING * 2) / colCount;

    int clickedRow = (y - PADDING) / ITEM_HEIGHT;
    int clickedCol = (x - PADDING) / colWidth;

    if (clickedCol >= 0 && clickedCol < colCount && clickedRow >= 0) {
        size_t trackIdx = clickedRow * colCount + clickedCol;
        FluyerLog::Info("UI", "Clicked track index " + std::to_string(trackIdx));
        if (trackIdx < m_trackCount && m_libraryService) {
            m_libraryService->PlayTrack(trackIdx);
        }
    }
    evt.Skip();
}

void MusicListCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);

    dc.SetBackground(wxBrush(wxColour(14, 14, 14)));
    dc.Clear();

    if (m_trackCount == 0 || !m_libraryService) return;

    int viewW = GetClientSize().GetWidth();
    int viewH = GetClientSize().GetHeight();
    int startPixelY = m_scrollOffsetY;
    int endPixelY = startPixelY + viewH;

    int colCount = std::max(1, (viewW - PADDING * 2) / MIN_COL_WIDTH);
    int colWidth = (viewW - PADDING * 2) / colCount;

    int startRow = std::max(0, (startPixelY - PADDING) / ITEM_HEIGHT);
    int endRow = (endPixelY - PADDING) / ITEM_HEIGHT + 1;

    double scaleFactor = GetContentScaleFactor();
    wxFont titleFont = wxFontInfo(wxSize(0, 13)).Bold().Family(wxFONTFAMILY_DEFAULT);
    wxFont subFont = wxFontInfo(wxSize(0, 11)).Family(wxFONTFAMILY_DEFAULT);

    for (int row = startRow; row <= endRow; ++row) {
        for (int col = 0; col < colCount; ++col) {
            size_t idx = row * colCount + col;
            if (idx >= m_trackCount) break;

            int cellX = PADDING + col * colWidth;
            int cellY = PADDING + row * ITEM_HEIGHT - m_scrollOffsetY;

            Track track = m_libraryService->GetTrack(idx);
            wxBitmap thumb;
            if (m_imageService) {
                thumb = m_imageService->GetTrackImage(idx, THUMB_SIZE, scaleFactor);
            }

            // Card background
            dc.SetBrush(wxBrush(wxColour(22, 22, 22)));
            dc.SetPen(wxPen(wxColour(32, 32, 32)));
            dc.DrawRoundedRectangle(cellX, cellY, colWidth - 8, ITEM_HEIGHT - 6, 6.0);

            // Thumbnail
            int thumbX = cellX + 6;
            int thumbY = cellY + 4;
            if (thumb.IsOk()) {
                dc.DrawBitmap(thumb, thumbX, thumbY, false);
            } else {
                dc.SetBrush(wxBrush(wxColour(38, 38, 38)));
                dc.SetPen(wxPen(wxColour(50, 50, 50)));
                dc.DrawRoundedRectangle(thumbX, thumbY, THUMB_SIZE, THUMB_SIZE, 4.0);
            }

            // Track metadata text
            int textX = thumbX + THUMB_SIZE + 8;
            int textMaxW = colWidth - THUMB_SIZE - 60;

            dc.SetFont(titleFont);
            dc.SetTextForeground(wxColour(235, 235, 235));
            wxString tStr = wxString::FromUTF8(track.title);
            if (dc.GetTextExtent(tStr).GetWidth() > textMaxW && tStr.length() > 20) {
                tStr = tStr.substr(0, 18) + "...";
            }
            dc.DrawText(tStr, textX, cellY + 8);

            dc.SetFont(subFont);
            dc.SetTextForeground(wxColour(150, 150, 150));
            wxString subStr = wxString::FromUTF8(track.artist) + " • " + wxString::FromUTF8(track.album);
            if (dc.GetTextExtent(subStr).GetWidth() > textMaxW && subStr.length() > 24) {
                subStr = subStr.substr(0, 22) + "...";
            }
            dc.DrawText(subStr, textX, cellY + 28);

            // Duration on right
            dc.SetTextForeground(wxColour(110, 110, 110));
            dc.DrawText(wxString::FromUTF8(track.FormatDuration()), cellX + colWidth - 48, cellY + 18);
        }
    }
}
