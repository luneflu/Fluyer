#include "AlbumListCtrl.h"
#include <wx/dcbuffer.h>
#include <wx/graphics.h>
#include <nlohmann/json.hpp>
#include <algorithm>

using json_t = nlohmann::json;

wxBEGIN_EVENT_TABLE(AlbumListCtrl, wxScrolledWindow)
    EVT_PAINT(AlbumListCtrl::OnPaint)
    EVT_SIZE(AlbumListCtrl::OnSize)
    EVT_LEFT_DOWN(AlbumListCtrl::OnLeftDown)
    EVT_MOUSEWHEEL(AlbumListCtrl::OnMouseWheel)
wxEND_EVENT_TABLE()

AlbumListCtrl::AlbumListCtrl(wxWindow* parent, wxWindowID id)
    : wxScrolledWindow(parent, id, wxDefaultPosition, wxDefaultSize, wxNO_BORDER | wxFULL_REPAINT_ON_RESIZE) {
    ShowScrollbars(wxSHOW_SB_NEVER, wxSHOW_SB_NEVER);
    SetBackgroundStyle(wxBG_STYLE_PAINT);
    SetBackgroundColour(wxColour(18, 18, 18));
}

void AlbumListCtrl::RefreshData(FluyerEngine* engine) {
    m_engine = engine;
    m_imageCache.clear();
    m_metaCache.clear();
    m_scrollOffsetX = 0;

    if (m_engine) {
        m_albumCount = fluyer_library_get_album_count(m_engine);
    } else {
        m_albumCount = 0;
    }

    Refresh();
}

void AlbumListCtrl::OnCoverLoaded(uintptr_t index) {
    m_imageCache.erase(index);
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
        // Handle click
    }
    evt.Skip();
}

void AlbumListCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);

    dc.SetBackground(wxBrush(wxColour(18, 18, 18)));
    dc.Clear();

    if (m_albumCount == 0 || !m_engine) return;

    int viewW = GetClientSize().GetWidth();
    int startPixelX = m_scrollOffsetX;
    int endPixelX = startPixelX + viewW;

    int startIdx = std::max(0, (startPixelX - ITEM_SPACING) / (ITEM_WIDTH + ITEM_SPACING));
    int endIdx = std::min(static_cast<int>(m_albumCount) - 1, (endPixelX - ITEM_SPACING) / (ITEM_WIDTH + ITEM_SPACING) + 1);

    for (int i = startIdx; i <= endIdx; ++i) {
        int itemX = ITEM_SPACING + i * (ITEM_WIDTH + ITEM_SPACING) - m_scrollOffsetX;
        int itemY = 8;

        // Metadata cache check
        if (m_metaCache.find(i) == m_metaCache.end()) {
            char* json = fluyer_library_get_album_json(m_engine, i);
            wxString albumName = "Unknown Album";
            wxString artistName = "Unknown Artist";
            if (json) {
                try {
                    auto j = json_t::parse(json);
                    if (j.is_array() && !j.empty()) {
                        const auto& first = j[0];
                        if (first.contains("album") && !first["album"].is_null()) {
                            albumName = first["album"].get<std::string>();
                        }
                        if (first.contains("albumArtist") && !first["albumArtist"].is_null()) {
                            artistName = first["albumArtist"].get<std::string>();
                        } else if (first.contains("artist") && !first["artist"].is_null()) {
                            artistName = first["artist"].get<std::string>();
                        }
                    }
                } catch (...) {}
                fluyer_string_free(json);
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
                    double scaleFactor = GetContentScaleFactor();
                    int targetW = static_cast<int>(COVER_SIZE * scaleFactor);
                    int targetH = static_cast<int>(COVER_SIZE * scaleFactor);
                    wxImage scaled = img.Scale(targetW, targetH, wxIMAGE_QUALITY_HIGH);
                    m_imageCache[i] = wxBitmap(scaled, -1, scaleFactor);
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
        wxFont titleFont = wxFontInfo(wxSize(0, 13)).Bold().Family(wxFONTFAMILY_DEFAULT);
        dc.SetFont(titleFont);
        
        wxString truncatedTitle = dc.GetTextExtent(meta.first).GetWidth() > COVER_SIZE ? meta.first.substr(0, 16) + "..." : meta.first;
        dc.DrawText(truncatedTitle, itemX, itemY + COVER_SIZE + 6);

        dc.SetTextForeground(wxColour(160, 160, 160));
        wxFont artistFont = wxFontInfo(wxSize(0, 11)).Family(wxFONTFAMILY_DEFAULT);
        dc.SetFont(artistFont);
        wxString truncatedArtist = dc.GetTextExtent(meta.second).GetWidth() > COVER_SIZE ? meta.second.substr(0, 18) + "..." : meta.second;
        dc.DrawText(truncatedArtist, itemX, itemY + COVER_SIZE + 24);
    }
}
