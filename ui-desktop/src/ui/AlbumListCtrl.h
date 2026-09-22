#pragma once
#include <wx/wx.h>
#include <functional>
#include "services/LibraryService.h"
#include "services/ImageService.h"

class AlbumListCtrl : public wxScrolledWindow {
public:
    AlbumListCtrl(wxWindow* parent, LibraryService* libraryService, ImageService* imageService, wxWindowID id = wxID_ANY);

    void RefreshData();
    void OnCoverLoaded(uintptr_t index);

    void SetSelectedAlbum(int index);
    int GetSelectedAlbum() const { return m_selectedAlbumIndex; }
    void SetOnAlbumSelected(std::function<void(int)> cb) { m_onAlbumSelected = std::move(cb); }

private:
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnScroll(wxScrollWinEvent& evt);
    void OnLeftDown(wxMouseEvent& evt);
    void OnLeftDClick(wxMouseEvent& evt);
    void OnMouseWheel(wxMouseEvent& evt);

    LibraryService* m_libraryService = nullptr;
    ImageService* m_imageService = nullptr;
    uintptr_t m_albumCount = 0;
    int m_scrollOffsetX = 0;
    int m_selectedAlbumIndex = -1;
    std::function<void(int)> m_onAlbumSelected;

    int GetItemWidth() const;
    int GetItemHeight() const;

    wxSize DoGetBestClientSize() const override;

    wxDECLARE_EVENT_TABLE();
};
