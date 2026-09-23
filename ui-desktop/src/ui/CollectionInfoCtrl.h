#pragma once
#include <wx/wx.h>
#include <wx/bmpbuttn.h>
#include <functional>
#include "services/LibraryService.h"
#include "models/Album.h"

// ponytail: single-album collection header; extend when folder/playlist collection types needed
class CollectionInfoCtrl : public wxPanel {
public:
    CollectionInfoCtrl(wxWindow* parent, LibraryService* libraryService, wxWindowID id = wxID_ANY);

    void SetAlbum(int albumIndex);
    void ClearAlbum();
    int GetAlbumIndex() const { return m_albumIndex; }

    void SetOnBack(std::function<void()> cb) { m_onBack = std::move(cb); }

    wxSize DoGetBestClientSize() const override { return wxSize(-1, 46); }

private:
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnBack(wxCommandEvent& evt);
    void OnPlay(wxCommandEvent& evt);
    void OnQueue(wxCommandEvent& evt);
    void OnShuffle(wxCommandEvent& evt);

    LibraryService* m_libraryService = nullptr;
    int m_albumIndex = -1;
    Album m_album;

    wxStaticText* m_lblInfo = nullptr;
    wxBitmapButton* m_btnBack = nullptr;
    wxBitmapButton* m_btnPlay = nullptr;
    wxBitmapButton* m_btnQueue = nullptr;
    wxBitmapButton* m_btnShuffle = nullptr;

    std::function<void()> m_onBack;

    wxDECLARE_EVENT_TABLE();
};
