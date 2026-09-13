#include "MainFrame.h"
#include <wx/dirdlg.h>

enum {
    ID_OPEN_FOLDER = 1001,
};

wxBEGIN_EVENT_TABLE(MainFrame, wxFrame)
    EVT_MENU(ID_OPEN_FOLDER, MainFrame::OnOpenFolder)
    EVT_MENU(wxID_EXIT, MainFrame::OnExit)
wxEND_EVENT_TABLE()

MainFrame::MainFrame(FluyerEngine* engine)
    : wxFrame(nullptr, wxID_ANY, "Fluyer Native", wxDefaultPosition, wxSize(900, 700)),
      m_engine(engine) {
    
    // Menu bar
    wxMenuBar* menuBar = new wxMenuBar();
    wxMenu* fileMenu = new wxMenu();
    fileMenu->Append(ID_OPEN_FOLDER, "&Open Music Folder...\tCtrl+O");
    fileMenu->AppendSeparator();
    fileMenu->Append(wxID_EXIT, "E&xit\tCtrl+Q");
    menuBar->Append(fileMenu, "&File");
    SetMenuBar(menuBar);

    // Splitter window: Albums top, Music bottom
    wxSplitterWindow* splitter = new wxSplitterWindow(this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxSP_LIVE_UPDATE | wxSP_3D);
    splitter->SetMinimumPaneSize(150);

    // Top: Album list container
    wxPanel* topPanel = new wxPanel(splitter);
    wxBoxSizer* topSizer = new wxBoxSizer(wxVERTICAL);

    m_albumList = new AlbumListCtrl(topPanel);
    topSizer->Add(m_albumList, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    topPanel->SetSizer(topSizer);

    // Bottom: Music list container
    wxPanel* bottomPanel = new wxPanel(splitter);
    wxBoxSizer* bottomSizer = new wxBoxSizer(wxVERTICAL);

    m_musicList = new MusicListCtrl(bottomPanel);
    bottomSizer->Add(m_musicList, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);
    bottomPanel->SetSizer(bottomSizer);

    splitter->SplitHorizontally(topPanel, bottomPanel, 220);

    RefreshViews();
}

MainFrame::~MainFrame() {
}

void MainFrame::RefreshViews() {
    if (m_albumList) m_albumList->RefreshData(m_engine);
    if (m_musicList) m_musicList->RefreshData(m_engine);
}

void MainFrame::OnOpenFolder(wxCommandEvent& WXUNUSED(evt)) {
    wxDirDialog dlg(this, "Choose music directory to scan", "", wxDD_DEFAULT_STYLE | wxDD_DIR_MUST_EXIST);
    if (dlg.ShowModal() == wxID_OK) {
        wxString path = dlg.GetPath();
        const char* c_path = path.c_str();
        const char* paths[] = { c_path };
        fluyer_library_scan(m_engine, paths, 1);
    }
}

void MainFrame::OnExit(wxCommandEvent& WXUNUSED(evt)) {
    Close(true);
}
