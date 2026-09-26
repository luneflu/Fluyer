#include "MainFrame.h"
#include <wx/dirdlg.h>
#include <algorithm>
#include <cmath>
#include <memory>

#ifdef __WXOSX__
#include "MainFrameNative.h"
#endif

enum {
    ID_OPEN_FOLDER = 1001,
    ID_TIMER_BG_ANIM = 1002,
};

wxBEGIN_EVENT_TABLE(MainFrame, wxFrame)
    EVT_MENU(ID_OPEN_FOLDER, MainFrame::OnOpenFolder)
    EVT_MENU(wxID_EXIT, MainFrame::OnExit)
    EVT_PAINT(MainFrame::OnPaint)
    EVT_SIZE(MainFrame::OnSize)
    EVT_TIMER(ID_TIMER_BG_ANIM, MainFrame::OnAnimTimer)
wxEND_EVENT_TABLE()

MainFrame::MainFrame(LibraryService* libraryService, PlayerService* playerService, ImageService* imageService, FluyerEngine* engine)
    : wxFrame(nullptr, wxID_ANY, "Fluyer Native", wxDefaultPosition, wxSize(960, 720)),
      m_libraryService(libraryService),
      m_playerService(playerService),
      m_imageService(imageService),
      m_engine(engine) {

    SetBackgroundStyle(wxBG_STYLE_PAINT);
    SetBackgroundColour(wxColour(18, 18, 22));
    m_animTimer.SetOwner(this, ID_TIMER_BG_ANIM);
    
#ifdef __WXOSX__
    // Initialize native CALayer background
    MainFrameNative::InitializeBackgroundLayer((void*)GetHandle());
#endif
    
    // Menu bar
    wxMenuBar* menuBar = new wxMenuBar();
    wxMenu* fileMenu = new wxMenu();
    fileMenu->Append(ID_OPEN_FOLDER, "&Open Music Folder...\tCtrl+O");
    fileMenu->AppendSeparator();
    fileMenu->Append(wxID_EXIT, "E&xit\tCtrl+Q");
    menuBar->Append(fileMenu, "&File");
    SetMenuBar(menuBar);

    wxBoxSizer* rootSizer = new wxBoxSizer(wxVERTICAL);

    // Top: Album list
    m_albumList = new AlbumListCtrl(this, m_libraryService, m_imageService);

    // Collection Info (hidden by default)
    m_collectionInfo = new CollectionInfoCtrl(this, m_libraryService);
    m_collectionInfo->Show(false);

    // Middle: Music list
    m_musicList = new MusicListCtrl(this, m_libraryService, m_imageService);

    // Bottom: Player bar
    m_playerBar = new PlayerBarCtrl(this, m_playerService, m_imageService);

    // Fullscreen Play/Lyrics view (hidden by default)
    m_playView = new PlayViewCtrl(this, m_playerService, m_imageService);
    m_playView->Show(false);

    rootSizer->Add(m_albumList, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    rootSizer->Add(m_collectionInfo, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    rootSizer->Add(m_musicList, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    rootSizer->Add(m_playerBar, 0, wxEXPAND);
    rootSizer->Add(m_playView, 1, wxEXPAND);
    SetSizer(rootSizer);

    m_playerBar->SetOnCoverClicked([this]() {
        ShowPlayView(true);
    });

    m_playView->SetOnBack([this]() {
        ShowPlayView(false);
    });

    m_albumList->SetOnAlbumSelected([this](int index) {
        if (index >= 0) {
            m_collectionInfo->SetAlbum(index);
            m_musicList->SetAlbumFilter(index);
        } else {
            m_collectionInfo->ClearAlbum();
            m_musicList->SetAlbumFilter(-1);
        }
        Layout();
    });

    m_collectionInfo->SetOnBack([this]() {
        m_albumList->SetSelectedAlbum(-1);
        m_collectionInfo->ClearAlbum();
        m_musicList->SetAlbumFilter(-1);
        Layout();
    });

    RefreshViews();
    UpdateBackground(true);
}

void MainFrame::ShowPlayView(bool show) {
    m_showingPlayView = show;
    if (show) {
        m_albumList->Show(false);
        m_collectionInfo->Show(false);
        m_musicList->Show(false);
        m_playerBar->Show(false);
        m_playView->Show(true);
        m_playView->UpdateTrack();
        m_playView->UpdateState();
    } else {
        m_playView->Show(false);
        if (m_collectionInfo && m_collectionInfo->GetAlbumIndex() >= 0) {
            m_collectionInfo->Show(true);
        } else {
            m_albumList->Show(true);
        }
        m_musicList->Show(true);
        m_playerBar->Show(true);
    }
    Layout();
}

MainFrame::~MainFrame() {
    if (m_animTimer.IsRunning()) {
        m_animTimer.Stop();
    }
#ifdef __WXOSX__
    MainFrameNative::CleanupBackgroundLayer((void*)GetHandle());
#endif
}

void MainFrame::RefreshViews() {
    if (m_albumList) m_albumList->RefreshData();
    if (m_collectionInfo && m_collectionInfo->GetAlbumIndex() >= 0) {
        m_collectionInfo->SetAlbum(m_collectionInfo->GetAlbumIndex());
    }
    if (m_musicList) m_musicList->RefreshData();
}

void MainFrame::OnTrackCoverLoaded(uintptr_t index) {
    if (m_musicList) m_musicList->OnCoverLoaded(index);
    if (m_playerBar) m_playerBar->OnCoverLoaded();
    if (m_playView) m_playView->OnCoverLoaded();
    UpdateBackground(true);
}

void MainFrame::OnAlbumCoverLoaded(uintptr_t index) {
    if (m_albumList) m_albumList->OnCoverLoaded(index);
}

void MainFrame::OnPlayerStateChanged(const FluyerPlayerState& state) {
    if (m_playerService) m_playerService->UpdateState(state);
    if (m_playerBar) m_playerBar->UpdateState();
    if (m_playView) m_playView->UpdateState();
}

void MainFrame::OnTrackChanged(const std::string& jsonMeta, uintptr_t index) {
    if (m_playerService) m_playerService->UpdateTrack(jsonMeta, index);
    if (m_playerBar) m_playerBar->UpdateTrack();
    if (m_playView) m_playView->UpdateTrack();
    UpdateBackground(false);
}

void MainFrame::UpdateBackground(bool force) {
    if (!m_engine) return;

    std::string newPath;
    if (m_playerService) {
        Track curr = m_playerService->GetCurrentTrack();
        newPath = curr.path;
    }

    if (m_currentMusicPath == newPath && !force && m_isInitialized) {
        return;
    }
    m_currentMusicPath = newPath;

    wxSize clientSize = GetClientSize();
    int currentWidth = std::max(960, clientSize.x);
    int currentHeight = std::max(720, clientSize.y);

    bool animated = m_isInitialized;
    m_isInitialized = true;
    m_lastRenderedWidth = currentWidth;
    m_lastRenderedHeight = currentHeight;

#ifdef __WXOSX__
    GenerateBackgroundAsync(currentWidth, currentHeight, animated);
#endif
}

void MainFrame::OnAnimTimer(wxTimerEvent& WXUNUSED(evt)) {
    // Native CALayer handles animation, no-op
}

void MainFrame::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxPaintDC dc(this);
    // Native CALayer paints background, no-op
}

void MainFrame::OnSize(wxSizeEvent& evt) {
    evt.Skip();

    wxSize clientSize = GetClientSize();
    if (clientSize.x <= 0 || clientSize.y <= 0) return;

    if (m_lastRenderedWidth == 0 || m_lastRenderedHeight == 0) {
        m_lastRenderedWidth = clientSize.x;
        m_lastRenderedHeight = clientSize.y;
        return;
    }

    double widthDiff = std::abs(clientSize.x - m_lastRenderedWidth) / static_cast<double>(m_lastRenderedWidth);
    double heightDiff = std::abs(clientSize.y - m_lastRenderedHeight) / static_cast<double>(m_lastRenderedHeight);

    if (widthDiff >= 0.25 || heightDiff >= 0.25) {
#ifdef __WXOSX__
        GenerateBackgroundAsync(clientSize.x, clientSize.y, false);
#endif
        m_lastRenderedWidth = clientSize.x;
        m_lastRenderedHeight = clientSize.y;
    }
}

void MainFrame::OnOpenFolder(wxCommandEvent& WXUNUSED(evt)) {
    wxDirDialog dlg(this, "Choose music directory to scan", "", wxDD_DEFAULT_STYLE | wxDD_DIR_MUST_EXIST);
    if (dlg.ShowModal() == wxID_OK) {
        wxString path = dlg.GetPath();
        if (m_libraryService) {
            m_libraryService->ScanDirectories({ path.ToStdString() });
        }
    }
}

void MainFrame::OnExit(wxCommandEvent& WXUNUSED(evt)) {
    Close(true);
}

void MainFrame::GenerateBackgroundAsync(int width, int height, bool animated) {
    if (m_isGenerating.exchange(true)) {
        // Already running — store the latest intent; the callback will pick it up.
        m_pendingGen = {width, height, animated, true};
        return;
    }

    FluyerEngine* engine = m_engine;
    std::thread([this, engine, width, height, animated]() {
        uint32_t texW = 0, texH = 0;
        uintptr_t len = 0;
        uint8_t* raw = fluyer_player_generate_background(engine, width, height, &texW, &texH, &len);

        if (raw && len > 0 && texW > 0 && texH > 0) {
            wxTheApp->CallAfter([this, raw, texW, texH, len, animated]() {
                SetNativeBackground(raw, texW, texH, animated);
                fluyer_bytes_free(raw, len);
                m_isGenerating = false;
                // Fire pending if there was a newer request
                if (m_pendingGen.valid) {
                    PendingGen p = m_pendingGen;
                    m_pendingGen.valid = false;
                    GenerateBackgroundAsync(p.width, p.height, p.animated);
                }
            });
        } else {
            wxTheApp->CallAfter([this, animated]() {
                uint8_t fallback[4] = {20, 20, 26, 255};
                SetNativeBackground(fallback, 1, 1, animated);
                m_isGenerating = false;
                if (m_pendingGen.valid) {
                    PendingGen p = m_pendingGen;
                    m_pendingGen.valid = false;
                    GenerateBackgroundAsync(p.width, p.height, p.animated);
                }
            });
        }
    }).detach();
}

void MainFrame::SetNativeBackground(uint8_t* data, uint32_t width, uint32_t height, bool animated) {
#ifdef __WXOSX__
    MainFrameNative::SetBackgroundImage((void*)GetHandle(), data, width, height, animated);
#endif
}
