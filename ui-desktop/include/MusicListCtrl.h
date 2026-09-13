#pragma once
#include <wx/wx.h>
#include <wx/mstream.h>
#include <unordered_map>
#include <string>
#include "fluyer_core.h"

struct MusicTrackItem {
    wxString title;
    wxString artist;
    wxString album;
    wxString duration;
};

class MusicListCtrl : public wxScrolledWindow {
public:
    MusicListCtrl(wxWindow* parent, wxWindowID id = wxID_ANY);
    void RefreshData(FluyerEngine* engine);
    void OnCoverLoaded(uintptr_t index);

private:
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnScroll(wxScrollWinEvent& evt);
    void OnLeftDClick(wxMouseEvent& evt);

    void OnMouseWheel(wxMouseEvent& evt);

    FluyerEngine* m_engine = nullptr;
    uintptr_t m_trackCount = 0;
    int m_scrollOffsetY = 0;

    std::unordered_map<uintptr_t, wxBitmap> m_imageCache;
    std::unordered_map<uintptr_t, MusicTrackItem> m_metaCache;

    static constexpr int ITEM_HEIGHT = 58;
    static constexpr int MIN_COL_WIDTH = 280;
    static constexpr int THUMB_SIZE = 44;
    static constexpr int PADDING = 8;

    wxDECLARE_EVENT_TABLE();
};
