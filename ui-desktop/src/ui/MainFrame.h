#pragma once
#include <wx/wx.h>
#include <wx/splitter.h>
#include "services/LibraryService.h"
#include "services/PlayerService.h"
#include "services/ImageService.h"
#include "AlbumListCtrl.h"
#include "CollectionInfoCtrl.h"
#include "MusicListCtrl.h"
#include "PlayerBarCtrl.h"

class MainFrame : public wxFrame {
public:
    MainFrame(LibraryService* libraryService, PlayerService* playerService, ImageService* imageService);
    ~MainFrame();

    void RefreshViews();
    void OnTrackCoverLoaded(uintptr_t index);
    void OnAlbumCoverLoaded(uintptr_t index);
    void OnPlayerStateChanged(const FluyerPlayerState& state);
    void OnTrackChanged(const std::string& jsonMeta, uintptr_t index);

private:
    void OnOpenFolder(wxCommandEvent& evt);
    void OnExit(wxCommandEvent& evt);

    LibraryService* m_libraryService;
    PlayerService* m_playerService;
    ImageService* m_imageService;

    AlbumListCtrl* m_albumList = nullptr;
    CollectionInfoCtrl* m_collectionInfo = nullptr;
    MusicListCtrl* m_musicList = nullptr;
    PlayerBarCtrl* m_playerBar = nullptr;

    wxDECLARE_EVENT_TABLE();
};
