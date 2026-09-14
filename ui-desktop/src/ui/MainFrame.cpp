#include "MainFrame.h"
#include <wx/dirdlg.h>

enum {
    ID_OPEN_FOLDER = 1001,
};

wxBEGIN_EVENT_TABLE(MainFrame, wxFrame)
    EVT_MENU(ID_OPEN_FOLDER, MainFrame::OnOpenFolder)
    EVT_MENU(wxID_EXIT, MainFrame::OnExit)
wxEND_EVENT_TABLE()

MainFrame::MainFrame(LibraryService* libraryService, PlayerService* playerService, ImageService* imageService)
    : wxFrame(nullptr, wxID_ANY, "Fluyer Native", wxDefaultPosition, wxSize(960, 720)),
      m_libraryService(libraryService),
      m_playerService(playerService),
      m_imageService(imageService) {
    
    // Menu bar
    wxMenuBar* menuBar = new wxMenuBar();
    wxMenu* fileMenu = new wxMenu();
    fileMenu->Append(ID_OPEN_FOLDER, "&Open Music Folder...\tCtrl+O");
    fileMenu->AppendSeparator();
    fileMenu->Append(wxID_EXIT, "E&xit\tCtrl+Q");
    menuBar->Append(fileMenu, "&File");
    SetMenuBar(menuBar);

    wxBoxSizer* rootSizer = new wxBoxSizer(wxVERTICAL);

    // Splitter window: Albums top, Music bottom
    wxSplitterWindow* splitter = new wxSplitterWindow(this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxSP_LIVE_UPDATE | wxSP_3D);
    splitter->SetMinimumPaneSize(150);

    // Top: Album list container
    wxPanel* topPanel = new wxPanel(splitter);
    wxBoxSizer* topSizer = new wxBoxSizer(wxVERTICAL);

    m_albumList = new AlbumListCtrl(topPanel, m_libraryService, m_imageService);
    topSizer->Add(m_albumList, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    topPanel->SetSizer(topSizer);

    // Bottom: Music list container
    wxPanel* bottomPanel = new wxPanel(splitter);
    wxBoxSizer* bottomSizer = new wxBoxSizer(wxVERTICAL);

    m_musicList = new MusicListCtrl(bottomPanel, m_libraryService, m_imageService);
    bottomSizer->Add(m_musicList, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    bottomPanel->SetSizer(bottomSizer);

    splitter->SplitHorizontally(topPanel, bottomPanel, 220);

    // Player bar below music list
    m_playerBar = new PlayerBarCtrl(this, m_playerService, m_imageService);

    rootSizer->Add(splitter, 1, wxEXPAND);
    rootSizer->Add(m_playerBar, 0, wxEXPAND);
    SetSizer(rootSizer);

    RefreshViews();
}

MainFrame::~MainFrame() = default;

void MainFrame::RefreshViews() {
    if (m_albumList) m_albumList->RefreshData();
    if (m_musicList) m_musicList->RefreshData();
}

void MainFrame::OnTrackCoverLoaded(uintptr_t index) {
    if (m_musicList) m_musicList->OnCoverLoaded(index);
    if (m_playerBar) m_playerBar->OnCoverLoaded();
}

void MainFrame::OnAlbumCoverLoaded(uintptr_t index) {
    if (m_albumList) m_albumList->OnCoverLoaded(index);
}

void MainFrame::OnPlayerStateChanged(const FluyerPlayerState& state) {
    if (m_playerService) m_playerService->UpdateState(state);
    if (m_playerBar) m_playerBar->UpdateState();
}

void MainFrame::OnTrackChanged(const std::string& jsonMeta, uintptr_t index) {
    if (m_playerService) m_playerService->UpdateTrack(jsonMeta, index);
    if (m_playerBar) m_playerBar->UpdateTrack();
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
