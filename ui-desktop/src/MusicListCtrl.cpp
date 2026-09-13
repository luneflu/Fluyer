#include "MusicListCtrl.h"
#include <wx/dcbuffer.h>
#include <wx/graphics.h>
#include <algorithm>
#include <iomanip>
#include <sstream>

wxBEGIN_EVENT_TABLE(MusicListCtrl, wxScrolledWindow)
    EVT_PAINT(MusicListCtrl::OnPaint)
    EVT_SIZE(MusicListCtrl::OnSize)
    EVT_SCROLLWIN(MusicListCtrl::OnScroll)
    EVT_LEFT_DCLICK(MusicListCtrl::OnLeftDClick)
wxEND_EVENT_TABLE()

MusicListCtrl::MusicListCtrl(wxWindow* parent, wxWindowID id)
    : wxScrolledWindow(parent, id, wxDefaultPosition, wxDefaultSize, wxVSCROLL | wxFULL_REPAINT_ON_RESIZE) {
    SetBackgroundStyle(wxBG_STYLE_PAINT);
    SetBackgroundColour(wxColour(14, 14, 14));
}

void MusicListCtrl::RefreshData(FluyerEngine* engine) {
    m_engine = engine;
    m_imageCache.clear();
    m_metaCache.clear();

    if (m_engine) {
        m_trackCount = fluyer_library_get_count(m_engine);
    } else {
        m_trackCount = 0;
    }

    int clientW = GetClientSize().GetWidth();
    int colCount = std::max(1, (clientW - PADDING * 2) / MIN_COL_WIDTH);
    int totalRows = (static_cast<int>(m_trackCount) + colCount - 1) / colCount;
    int totalHeight = totalRows * ITEM_HEIGHT + PADDING * 2;

    SetVirtualSize(clientW, totalHeight);
    SetScrollRate(0, ITEM_HEIGHT / 2);

    Refresh();
}

void MusicListCtrl::OnSize(wxSizeEvent& evt) {
    int clientW = evt.GetSize().GetWidth();
    int colCount = std::max(1, (clientW - PADDING * 2) / MIN_COL_WIDTH);
    int totalRows = (static_cast<int>(m_trackCount) + colCount - 1) / colCount;
    int totalHeight = totalRows * ITEM_HEIGHT + PADDING * 2;

    SetVirtualSize(clientW, totalHeight);
    Refresh();
    evt.Skip();
}

void MusicListCtrl::OnScroll(wxScrollWinEvent& evt) {
    Refresh();
    evt.Skip();
}

void MusicListCtrl::OnLeftDClick(wxMouseEvent& evt) {
    int x, y;
    CalcUnscrolledPosition(evt.GetX(), evt.GetY(), &x, &y);
    int clientW = GetClientSize().GetWidth();
    int colCount = std::max(1, (clientW - PADDING * 2) / MIN_COL_WIDTH);
    int colWidth = (clientW - PADDING * 2) / colCount;

    int clickedRow = (y - PADDING) / ITEM_HEIGHT;
    int clickedCol = (x - PADDING) / colWidth;

    if (clickedCol >= 0 && clickedCol < colCount && clickedRow >= 0) {
        size_t trackIdx = clickedRow * colCount + clickedCol;
        if (trackIdx < m_trackCount && m_engine) {
            fluyer_library_play_index(m_engine, trackIdx);
        }
    }
    evt.Skip();
}

void MusicListCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);
    DoPrepareDC(dc);

    dc.SetBackground(wxBrush(wxColour(14, 14, 14)));
    dc.Clear();

    if (m_trackCount == 0 || !m_engine) return;

    int viewX, viewY, viewW, viewH;
    GetViewStart(&viewX, &viewY);
    int xUnit, yUnit;
    GetScrollPixelsPerUnit(&xUnit, &yUnit);
    int startPixelY = viewY * yUnit;
    GetClientSize(&viewW, &viewH);
    int endPixelY = startPixelY + viewH;

    int colCount = std::max(1, (viewW - PADDING * 2) / MIN_COL_WIDTH);
    int colWidth = (viewW - PADDING * 2) / colCount;

    int startRow = std::max(0, (startPixelY - PADDING) / ITEM_HEIGHT);
    int endRow = (endPixelY - PADDING) / ITEM_HEIGHT + 1;

    wxFont titleFont(10, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD);
    wxFont subFont(9, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_NORMAL);

    for (int row = startRow; row <= endRow; ++row) {
        for (int col = 0; col < colCount; ++col) {
            size_t idx = row * colCount + col;
            if (idx >= m_trackCount) break;

            int cellX = PADDING + col * colWidth;
            int cellY = PADDING + row * ITEM_HEIGHT;

            // Metadata cache
            if (m_metaCache.find(idx) == m_metaCache.end()) {
                char* json = fluyer_library_get_track_json(m_engine, idx);
                MusicTrackItem item = { "Unknown Title", "Unknown Artist", "Unknown Album", "0:00" };
                if (json) {
                    std::string raw(json);
                    fluyer_string_free(json);

                    size_t tPos = raw.find("\"title\":\"");
                    if (tPos != std::string::npos) {
                        size_t s = tPos + 9;
                        size_t e = raw.find("\"", s);
                        if (e != std::string::npos) item.title = raw.substr(s, e - s);
                    }
                    size_t aPos = raw.find("\"artist\":\"");
                    if (aPos != std::string::npos) {
                        size_t s = aPos + 10;
                        size_t e = raw.find("\"", s);
                        if (e != std::string::npos) item.artist = raw.substr(s, e - s);
                    }
                    size_t albPos = raw.find("\"album\":\"");
                    if (albPos != std::string::npos) {
                        size_t s = albPos + 9;
                        size_t e = raw.find("\"", s);
                        if (e != std::string::npos) item.album = raw.substr(s, e - s);
                    }
                    size_t dPos = raw.find("\"duration\":");
                    if (dPos != std::string::npos) {
                        size_t s = dPos + 11;
                        size_t e = raw.find_first_of(",}", s);
                        if (e != std::string::npos) {
                            std::string num = raw.substr(s, e - s);
                            if (num != "null") {
                                try {
                                    uint64_t dur_ms = std::stoull(num);
                                    uint64_t total_sec = dur_ms / 1000;
                                    std::ostringstream ss;
                                    ss << (total_sec / 60) << ":" << std::setw(2) << std::setfill('0') << (total_sec % 60);
                                    item.duration = ss.str();
                                } catch (...) {}
                            }
                        }
                    }
                }
                m_metaCache[idx] = item;
            }

            // Image cache
            if (m_imageCache.find(idx) == m_imageCache.end()) {
                uintptr_t imgLen = 0;
                uint8_t* bytes = fluyer_library_get_track_image(m_engine, idx, &imgLen);
                if (bytes && imgLen > 0) {
                    wxMemoryInputStream stream(bytes, imgLen);
                    wxImage img(stream);
                    if (img.IsOk()) {
                        wxImage scaled = img.Scale(THUMB_SIZE, THUMB_SIZE, wxIMAGE_QUALITY_HIGH);
                        m_imageCache[idx] = wxBitmap(scaled);
                    }
                    fluyer_bytes_free(bytes, imgLen);
                }
            }

            // Draw track card background
            dc.SetBrush(wxBrush(wxColour(22, 22, 22)));
            dc.SetPen(wxPen(wxColour(32, 32, 32)));
            dc.DrawRoundedRectangle(cellX, cellY, colWidth - 8, ITEM_HEIGHT - 6, 6.0);

            // Draw thumbnail
            int thumbX = cellX + 6;
            int thumbY = cellY + 4;
            if (m_imageCache.find(idx) != m_imageCache.end() && m_imageCache[idx].IsOk()) {
                dc.DrawBitmap(m_imageCache[idx], thumbX, thumbY, false);
            } else {
                dc.SetBrush(wxBrush(wxColour(38, 38, 38)));
                dc.SetPen(wxPen(wxColour(50, 50, 50)));
                dc.DrawRoundedRectangle(thumbX, thumbY, THUMB_SIZE, THUMB_SIZE, 4.0);
            }

            // Draw track metadata text
            const auto& item = m_metaCache[idx];
            int textX = thumbX + THUMB_SIZE + 8;
            int textMaxW = colWidth - THUMB_SIZE - 60;

            dc.SetFont(titleFont);
            dc.SetTextForeground(wxColour(235, 235, 235));
            wxString tStr = item.title;
            if (dc.GetTextExtent(tStr).GetWidth() > textMaxW && tStr.length() > 20) {
                tStr = tStr.substr(0, 18) + "...";
            }
            dc.DrawText(tStr, textX, cellY + 8);

            dc.SetFont(subFont);
            dc.SetTextForeground(wxColour(150, 150, 150));
            wxString subStr = item.artist + " • " + item.album;
            if (dc.GetTextExtent(subStr).GetWidth() > textMaxW && subStr.length() > 24) {
                subStr = subStr.substr(0, 22) + "...";
            }
            dc.DrawText(subStr, textX, cellY + 28);

            // Duration on right
            dc.SetTextForeground(wxColour(110, 110, 110));
            dc.DrawText(item.duration, cellX + colWidth - 48, cellY + 18);
        }
    }
}
