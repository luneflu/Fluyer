#include "CollectionInfoCtrl.h"
#include "common/Icons.h"
#include <wx/dcbuffer.h>
#include <wx/settings.h>

enum {
    ID_BTN_BACK = 3001,
    ID_BTN_PLAY,
    ID_BTN_QUEUE,
    ID_BTN_SHUFFLE,
};

wxBEGIN_EVENT_TABLE(CollectionInfoCtrl, wxPanel)
    EVT_PAINT(CollectionInfoCtrl::OnPaint)
    EVT_SIZE(CollectionInfoCtrl::OnSize)
    EVT_BUTTON(ID_BTN_BACK, CollectionInfoCtrl::OnBack)
    EVT_BUTTON(ID_BTN_PLAY, CollectionInfoCtrl::OnPlay)
    EVT_BUTTON(ID_BTN_QUEUE, CollectionInfoCtrl::OnQueue)
    EVT_BUTTON(ID_BTN_SHUFFLE, CollectionInfoCtrl::OnShuffle)
wxEND_EVENT_TABLE()

CollectionInfoCtrl::CollectionInfoCtrl(wxWindow* parent, LibraryService* libraryService, wxWindowID id)
    : wxPanel(parent, id, wxDefaultPosition, wxSize(-1, 42), wxNO_BORDER),
      m_libraryService(libraryService) {
    SetBackgroundStyle(wxBG_STYLE_PAINT);

    wxColour btnCol = wxSystemSettings::GetColour(wxSYS_COLOUR_BTNTEXT);
    wxSize iconSize(22, 22);

    wxBoxSizer* mainSizer = new wxBoxSizer(wxHORIZONTAL);

    m_lblInfo = new wxStaticText(this, wxID_ANY, "", wxDefaultPosition, wxDefaultSize, wxST_ELLIPSIZE_END);
    m_lblInfo->SetFont(wxFontInfo(wxSize(0, 13)).Weight(wxFONTWEIGHT_MEDIUM).Family(wxFONTFAMILY_DEFAULT));
    m_lblInfo->SetForegroundColour(btnCol);

    wxBoxSizer* btnSizer = new wxBoxSizer(wxHORIZONTAL);

    m_btnBack = new wxBitmapButton(this, ID_BTN_BACK, Icons::Get(Icons::Back, btnCol, iconSize), wxDefaultPosition, iconSize, wxBORDER_NONE);
    m_btnBack->SetToolTip("Back");
    m_btnBack->SetBitmapMargins(0, 0);

    m_btnPlay = new wxBitmapButton(this, ID_BTN_PLAY, Icons::Get(Icons::Play, btnCol, iconSize), wxDefaultPosition, iconSize, wxBORDER_NONE);
    m_btnPlay->SetToolTip("Play");
    m_btnPlay->SetBitmapMargins(0, 0);

    m_btnQueue = new wxBitmapButton(this, ID_BTN_QUEUE, Icons::Get(Icons::QueueMusic, btnCol, iconSize), wxDefaultPosition, iconSize, wxBORDER_NONE);
    m_btnQueue->SetToolTip("Add to Queue");
    m_btnQueue->SetBitmapMargins(0, 0);

    m_btnShuffle = new wxBitmapButton(this, ID_BTN_SHUFFLE, Icons::Get(Icons::Shuffle, btnCol, iconSize), wxDefaultPosition, iconSize, wxBORDER_NONE);
    m_btnShuffle->SetToolTip("Shuffle");
    m_btnShuffle->SetBitmapMargins(0, 0);

    btnSizer->Add(m_btnBack, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 8);
    btnSizer->Add(m_btnPlay, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 8);
    btnSizer->Add(m_btnQueue, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 8);
    btnSizer->Add(m_btnShuffle, 0, wxALIGN_CENTER_VERTICAL, 0);

    mainSizer->Add(m_lblInfo, 1, wxALIGN_CENTER_VERTICAL | wxLEFT, 16);
    mainSizer->Add(btnSizer, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 12);

    SetSizer(mainSizer);
}

void CollectionInfoCtrl::SetAlbum(int albumIndex) {
    m_albumIndex = albumIndex;
    if (m_libraryService && m_albumIndex >= 0) {
        m_album = m_libraryService->GetAlbum(m_albumIndex);
        m_lblInfo->SetLabel(wxString::FromUTF8(m_album.GetLabel()));
        Show(true);
    } else {
        ClearAlbum();
    }
    Refresh();
}

void CollectionInfoCtrl::ClearAlbum() {
    m_albumIndex = -1;
    m_album = Album{};
    m_lblInfo->SetLabel("");
    Show(false);
    Refresh();
}

void CollectionInfoCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);
    dc.SetBackground(wxBrush(GetParent() ? GetParent()->GetBackgroundColour() : GetBackgroundColour()));
    dc.Clear();

    if (!IsShown() || m_albumIndex < 0) return;

    wxRect rect = GetClientRect();
    rect.Deflate(2, 2);

    wxColour bg = wxSystemSettings::GetColour(wxSYS_COLOUR_BTNFACE);
    wxColour border = wxSystemSettings::GetColour(wxSYS_COLOUR_BTNSHADOW);
    dc.SetBrush(wxBrush(bg));
    dc.SetPen(wxPen(border));
    dc.DrawRoundedRectangle(rect, 6.0);
}

void CollectionInfoCtrl::OnSize(wxSizeEvent& evt) {
    Refresh();
    evt.Skip();
}

void CollectionInfoCtrl::OnBack(wxCommandEvent& WXUNUSED(evt)) {
    ClearAlbum();
    if (m_onBack) {
        m_onBack();
    }
}

void CollectionInfoCtrl::OnPlay(wxCommandEvent& WXUNUSED(evt)) {
    if (m_libraryService && m_albumIndex >= 0) {
        m_libraryService->PlayAlbum(m_albumIndex);
    }
}

void CollectionInfoCtrl::OnQueue(wxCommandEvent& WXUNUSED(evt)) {
    if (m_libraryService && m_albumIndex >= 0) {
        m_libraryService->QueueAlbum(m_albumIndex);
    }
}

void CollectionInfoCtrl::OnShuffle(wxCommandEvent& WXUNUSED(evt)) {
    if (m_libraryService && m_albumIndex >= 0) {
        m_libraryService->ShuffleAlbum(m_albumIndex);
    }
}
