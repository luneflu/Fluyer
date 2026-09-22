#pragma once
#include <wx/wx.h>
#include "services/LibraryService.h"
#include "services/ImageService.h"

class MusicListCtrl : public wxScrolledWindow {
public:
    MusicListCtrl(wxWindow* parent, LibraryService* libraryService, ImageService* imageService, wxWindowID id = wxID_ANY);

    void RefreshData();
    void OnCoverLoaded(uintptr_t index);

    void SetAlbumFilter(int albumIndex);
    int GetAlbumFilter() const { return m_albumFilter; }

private:
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnScroll(wxScrollWinEvent& evt);
    void OnLeftDown(wxMouseEvent& evt);
    void OnMouseWheel(wxMouseEvent& evt);
    int GetColumnCount() const;

    LibraryService* m_libraryService = nullptr;
    ImageService* m_imageService = nullptr;
    uintptr_t m_trackCount = 0;
    int m_scrollOffsetY = 0;
    int m_albumFilter = -1;
    Album m_filteredAlbum;

    static constexpr int ITEM_HEIGHT = 58;
    static constexpr int MIN_COL_WIDTH = 280;
    static constexpr int THUMB_SIZE = 44;
    static constexpr int PADDING = 8;

    wxDECLARE_EVENT_TABLE();
};
