#pragma once
#include <wx/wx.h>
#include <wx/mstream.h>
#include <unordered_map>
#include <string>
#include "fluyer_core.h"

class AlbumListCtrl : public wxScrolledWindow {
public:
    AlbumListCtrl(wxWindow* parent, wxWindowID id = wxID_ANY);
    void RefreshData(FluyerEngine* engine);

private:
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnScroll(wxScrollWinEvent& evt);
    void OnLeftDown(wxMouseEvent& evt);

    void OnMouseWheel(wxMouseEvent& evt);

    FluyerEngine* m_engine = nullptr;
    uintptr_t m_albumCount = 0;
    int m_scrollOffsetX = 0;
    
    // Cached items
    std::unordered_map<uintptr_t, wxBitmap> m_imageCache;
    std::unordered_map<uintptr_t, std::pair<wxString, wxString>> m_metaCache;

    static constexpr int ITEM_WIDTH = 150;
    static constexpr int ITEM_HEIGHT = 190;
    static constexpr int ITEM_SPACING = 14;
    static constexpr int COVER_SIZE = 140;

    wxDECLARE_EVENT_TABLE();
};
