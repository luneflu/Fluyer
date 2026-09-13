#include "AlbumListCtrl.h"
#include <wx/dcbuffer.h>
#include <wx/graphics.h>
#include <algorithm>

wxBEGIN_EVENT_TABLE(AlbumListCtrl, wxScrolledWindow)
    EVT_PAINT(AlbumListCtrl::OnPaint)
    EVT_SIZE(AlbumListCtrl::OnSize)
    EVT_SCROLLWIN(AlbumListCtrl::OnScroll)
    EVT_LEFT_DOWN(AlbumListCtrl::OnLeftDown)
wxEND_EVENT_TABLE()

AlbumListCtrl::AlbumListCtrl(wxWindow* parent, wxWindowID id)
    : wxScrolledWindow(parent, id, wxDefaultPosition, wxDefaultSize, wxHSCROLL | wxFULL_REPAINT_ON_RESIZE) {
    SetBackgroundStyle(wxBG_STYLE_PAINT);
    SetBackgroundColour(wxColour(18, 18, 18));
}

void AlbumListCtrl::RefreshData(FluyerEngine* engine) {
    m_engine = engine;
    m_imageCache.clear();
    m_metaCache.clear();

    if (m_engine) {
        m_albumCount = fluyer_library_get_album_count(m_engine);
    } else {
        m_albumCount = 0;
    }

    int totalWidth = static_cast<int>(m_albumCount) * (ITEM_WIDTH + ITEM_SPACING) + ITEM_SPACING;
    SetVirtualSize(std::max(totalWidth, GetClientSize().GetWidth()), ITEM_HEIGHT);
    SetScrollRate(20, 0);

    Refresh();
}

void AlbumListCtrl::OnSize(wxSizeEvent& evt) {
    int totalWidth = static_cast<int>(m_albumCount) * (ITEM_WIDTH + ITEM_SPACING) + ITEM_SPACING;
    SetVirtualSize(std::max(totalWidth, GetClientSize().GetWidth()), ITEM_HEIGHT);
    Refresh();
    evt.Skip();
}

void AlbumListCtrl::OnScroll(wxScrollWinEvent& evt) {
    Refresh();
    evt.Skip();
}

void AlbumListCtrl::OnLeftDown(wxMouseEvent& evt) {
    int x, y;
    CalcUnscrolledPosition(evt.GetX(), evt.GetY(), &x, &y);
    int clickedIdx = (x - ITEM_SPACING) / (ITEM_WIDTH + ITEM_SPACING);
    if (clickedIdx >= 0 && clickedIdx < static_cast<int>(m_albumCount)) {
        // Play first track of album or select
    }
    evt.Skip();
}

void AlbumListCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);
    DoPrepareDC(dc);

    dc.SetBackground(wxBrush(wxColour(18, 18, 18)));
    dc.Clear();

    if (m_albumCount == 0 || !m_engine) return;

    int viewX, viewY, viewW, viewH;
    GetViewStart(&viewX, &viewY);
    int xUnit, yUnit;
    GetScrollPixelsPerUnit(&xUnit, &yUnit);
    int startPixelX = viewX * xUnit;
    GetClientSize(&viewW, &viewH);
    int endPixelX = startPixelX + viewW;

    int startIdx = std::max(0, (startPixelX - ITEM_SPACING) / (ITEM_WIDTH + ITEM_SPACING));
    int endIdx = std::min(static_cast<int>(m_albumCount) - 1, (endPixelX - ITEM_SPACING) / (ITEM_WIDTH + ITEM_SPACING) + 1);

    for (int i = startIdx; i <= endIdx; ++i) {
        int itemX = ITEM_SPACING + i * (ITEM_WIDTH + ITEM_SPACING);
        int itemY = 8;

        // Metadata cache check
        if (m_metaCache.find(i) == m_metaCache.end()) {
            char* json = fluyer_library_get_album_json(m_engine, i);
            wxString albumName = "Unknown Album";
            wxString artistName = "Unknown Artist";
            if (json) {
                std::string raw(json);
                fluyer_string_free(json);
                size_t albPos = raw.find("\"album\":\"");
                if (albPos != std::string::npos) {
                    size_t start = albPos + 9;
                    size_t end = raw.find("\"", start);
                    if (end != std::string::npos) albumName = raw.substr(start, end - start);
                }
                size_t artPos = raw.find("\"albumArtist\":\"");
                if (artPos == std::string::npos) artPos = raw.find("\"artist\":\"");
                if (artPos != std::string::npos) {
                    size_t start = raw.find(":", artPos) + 2;
                    size_t end = raw.find("\"", start);
                    if (end != std::string::npos) artistName = raw.substr(start, end - start);
                }
            }
            m_metaCache[i] = { albumName, artistName };
        }

        // Image cache check
        if (m_imageCache.find(i) == m_imageCache.end()) {
            uintptr_t imgLen = 0;
            uint8_t* bytes = fluyer_library_get_album_image(m_engine, i, &imgLen);
            if (bytes && imgLen > 0) {
                wxMemoryInputStream stream(bytes, imgLen);
                wxImage img(stream);
                if (img.IsOk()) {
                    wxImage scaled = img.Scale(COVER_SIZE, COVER_SIZE, wxIMAGE_QUALITY_HIGH);
                    m_imageCache[i] = wxBitmap(scaled);
                }
                fluyer_bytes_free(bytes, imgLen);
            }
        }

        // Draw cover or placeholder
        if (m_imageCache.find(i) != m_imageCache.end() && m_imageCache[i].IsOk()) {
            dc.DrawBitmap(m_imageCache[i], itemX, itemY, false);
        } else {
            dc.SetBrush(wxBrush(wxColour(32, 32, 32)));
            dc.SetPen(wxPen(wxColour(48, 48, 48)));
            dc.DrawRoundedRectangle(itemX, itemY, COVER_SIZE, COVER_SIZE, 8.0);
        }

        // Draw text
        const auto& meta = m_metaCache[i];
        dc.SetTextForeground(wxColour(240, 240, 240));
        wxFont titleFont(11, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD);
        dc.SetFont(titleFont);
        
        wxString truncatedTitle = dc.GetTextExtent(meta.first).GetWidth() > COVER_SIZE ? meta.first.substr(0, 16) + "..." : meta.first;
        dc.DrawText(truncatedTitle, itemX, itemY + COVER_SIZE + 6);

        dc.SetTextForeground(wxColour(160, 160, 160));
        wxFont artistFont(10, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_NORMAL);
        dc.SetFont(artistFont);
        wxString truncatedArtist = dc.GetTextExtent(meta.second).GetWidth() > COVER_SIZE ? meta.second.substr(0, 18) + "..." : meta.second;
        dc.DrawText(truncatedArtist, itemX, itemY + COVER_SIZE + 24);
    }
}
