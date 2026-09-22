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

    // Top: Album list
    m_albumList = new AlbumListCtrl(this, m_libraryService, m_imageService);

    // Collection Info (hidden by default)
    m_collectionInfo = new CollectionInfoCtrl(this, m_libraryService);
    m_collectionInfo->Show(false);

    // Middle: Music list
    m_musicList = new MusicListCtrl(this, m_libraryService, m_imageService);

    // Bottom: Player bar
    m_playerBar = new PlayerBarCtrl(this, m_playerService, m_imageService);

    rootSizer->Add(m_albumList, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    rootSizer->Add(m_collectionInfo, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    rootSizer->Add(m_musicList, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    rootSizer->Add(m_playerBar, 0, wxEXPAND);
    SetSizer(rootSizer);

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
}

MainFrame::~MainFrame() = default;

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
