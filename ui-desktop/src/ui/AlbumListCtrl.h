#pragma once
#include <wx/wx.h>
#include "services/LibraryService.h"
#include "services/ImageService.h"

class AlbumListCtrl : public wxScrolledWindow {
public:
    AlbumListCtrl(wxWindow* parent, LibraryService* libraryService, ImageService* imageService, wxWindowID id = wxID_ANY);

    void RefreshData();
    void OnCoverLoaded(uintptr_t index);

private:
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnScroll(wxScrollWinEvent& evt);
    void OnLeftDown(wxMouseEvent& evt);
    void OnMouseWheel(wxMouseEvent& evt);

    LibraryService* m_libraryService = nullptr;
    ImageService* m_imageService = nullptr;
    uintptr_t m_albumCount = 0;
    int m_scrollOffsetX = 0;

    static constexpr int ITEM_WIDTH = 150;
    static constexpr int ITEM_HEIGHT = 190;
    static constexpr int ITEM_SPACING = 14;
    static constexpr int COVER_SIZE = 140;

    wxDECLARE_EVENT_TABLE();
};
