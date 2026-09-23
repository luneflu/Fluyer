#pragma once
#include <wx/wx.h>
#include <wx/splitter.h>
#include <wx/timer.h>
#include "fluyer_core.h"
#include "services/LibraryService.h"
#include "services/PlayerService.h"
#include "services/ImageService.h"
#include "AlbumListCtrl.h"
#include "CollectionInfoCtrl.h"
#include "MusicListCtrl.h"
#include "PlayerBarCtrl.h"
#include <thread>
#include <atomic>

class MainFrame : public wxFrame {
public:
    MainFrame(LibraryService* libraryService, PlayerService* playerService, ImageService* imageService, FluyerEngine* engine = nullptr);
    ~MainFrame();

    void RefreshViews();
    void OnTrackCoverLoaded(uintptr_t index);
    void OnAlbumCoverLoaded(uintptr_t index);
    void OnPlayerStateChanged(const FluyerPlayerState& state);
    void OnTrackChanged(const std::string& jsonMeta, uintptr_t index);

    void UpdateBackground(bool force = false);

private:
    void OnOpenFolder(wxCommandEvent& evt);
    void OnExit(wxCommandEvent& evt);
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnAnimTimer(wxTimerEvent& evt);
    
    void GenerateBackgroundAsync(int width, int height, bool animated);
    void SetNativeBackground(uint8_t* data, uint32_t width, uint32_t height, bool animated);

    LibraryService* m_libraryService;
    PlayerService* m_playerService;
    ImageService* m_imageService;
    FluyerEngine* m_engine = nullptr;

    AlbumListCtrl* m_albumList = nullptr;
    CollectionInfoCtrl* m_collectionInfo = nullptr;
    MusicListCtrl* m_musicList = nullptr;
    PlayerBarCtrl* m_playerBar = nullptr;

    // Background state
    wxTimer m_animTimer;
    bool m_isInitialized = false;
    int m_lastRenderedWidth = 0;
    int m_lastRenderedHeight = 0;
    std::string m_currentMusicPath;
    std::atomic<bool> m_isGenerating{false};
    // Pending generation: if a new request arrives while one is in-flight,
    // store the latest args and fire it when the current one finishes.
    struct PendingGen { int width; int height; bool animated; bool valid = false; };
    PendingGen m_pendingGen;

    wxDECLARE_EVENT_TABLE();
};
