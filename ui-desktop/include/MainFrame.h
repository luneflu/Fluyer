#pragma once
#include <wx/wx.h>
#include <wx/splitter.h>
#include "fluyer_core.h"
#include "AlbumListCtrl.h"
#include "MusicListCtrl.h"

class MainFrame : public wxFrame {
public:
    MainFrame(FluyerEngine* engine);
    ~MainFrame();

    void RefreshViews();
    void OnTrackCoverLoaded(uintptr_t index);
    void OnAlbumCoverLoaded(uintptr_t index);

private:
    void OnOpenFolder(wxCommandEvent& evt);
    void OnExit(wxCommandEvent& evt);

    FluyerEngine* m_engine;
    AlbumListCtrl* m_albumList;
    MusicListCtrl* m_musicList;

    wxDECLARE_EVENT_TABLE();
};
