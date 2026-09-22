#include "PlayerBarCtrl.h"
#include "common/Icons.h"
#include <wx/dcbuffer.h>
#include <wx/settings.h>
#include <algorithm>

enum {
    ID_BTN_PLAY_PAUSE = 2001,
    ID_BTN_PREV,
    ID_BTN_NEXT,
    ID_BTN_SHUFFLE,
    ID_BTN_REPEAT,
    ID_SLIDER_SEEK,
    ID_SLIDER_VOL,
    ID_TIMER_PROGRESS
};

wxBEGIN_EVENT_TABLE(PlayerBarCtrl, wxPanel)
    EVT_PAINT(PlayerBarCtrl::OnPaint)
    EVT_SIZE(PlayerBarCtrl::OnSize)
    EVT_BUTTON(ID_BTN_PLAY_PAUSE, PlayerBarCtrl::OnPlayPause)
    EVT_BUTTON(ID_BTN_PREV, PlayerBarCtrl::OnPrev)
    EVT_BUTTON(ID_BTN_NEXT, PlayerBarCtrl::OnNext)
    EVT_BUTTON(ID_BTN_SHUFFLE, PlayerBarCtrl::OnShuffle)
    EVT_BUTTON(ID_BTN_REPEAT, PlayerBarCtrl::OnRepeat)
    EVT_SLIDER(ID_SLIDER_SEEK, PlayerBarCtrl::OnSeek)
    EVT_SLIDER(ID_SLIDER_VOL, PlayerBarCtrl::OnVolume)
    EVT_TIMER(ID_TIMER_PROGRESS, PlayerBarCtrl::OnTimer)
wxEND_EVENT_TABLE()

PlayerBarCtrl::PlayerBarCtrl(wxWindow* parent, PlayerService* playerService, ImageService* imageService, wxWindowID id)
    : wxPanel(parent, id, wxDefaultPosition, wxSize(-1, 108), wxNO_BORDER),
      m_playerService(playerService),
      m_imageService(imageService),
      m_progressTimer(this, ID_TIMER_PROGRESS) {
    SetBackgroundStyle(wxBG_STYLE_PAINT);

    wxBoxSizer* outerSizer = new wxBoxSizer(wxVERTICAL);

    wxColour grayCol = wxSystemSettings::GetColour(wxSYS_COLOUR_GRAYTEXT);
    wxColour btnCol = wxSystemSettings::GetColour(wxSYS_COLOUR_BTNTEXT);

    // 1. Progress Bar
    wxBoxSizer* progSizer = new wxBoxSizer(wxHORIZONTAL);
    m_lblTimePos = new wxStaticText(this, wxID_ANY, "0:00", wxDefaultPosition, wxSize(42, -1), wxALIGN_RIGHT);
    m_lblTimePos->SetForegroundColour(grayCol);
    m_lblTimePos->SetFont(wxFontInfo(wxSize(0, 10)).Family(wxFONTFAMILY_DEFAULT));

    m_seekSlider = new wxSlider(this, ID_SLIDER_SEEK, 0, 0, 1000, wxDefaultPosition, wxDefaultSize, wxSL_HORIZONTAL);

    m_lblTimeDur = new wxStaticText(this, wxID_ANY, "0:00", wxDefaultPosition, wxSize(42, -1), wxALIGN_LEFT);
    m_lblTimeDur->SetForegroundColour(grayCol);
    m_lblTimeDur->SetFont(wxFontInfo(wxSize(0, 10)).Family(wxFONTFAMILY_DEFAULT));

    progSizer->Add(m_lblTimePos, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    progSizer->Add(m_seekSlider, 1, wxALIGN_CENTER_VERTICAL);
    progSizer->Add(m_lblTimeDur, 0, wxALIGN_CENTER_VERTICAL | wxLEFT, 6);

    outerSizer->Add(progSizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxTOP, 10);
    outerSizer->AddSpacer(4);

    // 2. Control Pill Container
    m_pillPanel = new wxPanel(this, wxID_ANY);

    wxBoxSizer* pillSizer = new wxBoxSizer(wxHORIZONTAL);

    // Column 1: Playback Controls (Previous, Play/Pause, Next)
    wxBoxSizer* leftCtrlSizer = new wxBoxSizer(wxHORIZONTAL);
    m_btnPrev = new wxBitmapButton(m_pillPanel, ID_BTN_PREV, Icons::Get(Icons::Prev, btnCol, wxSize(24, 24)), wxDefaultPosition, wxSize(24, 24), wxBORDER_NONE);
    m_btnPrev->SetBitmapMargins(0, 0);

    m_btnPlayPause = new wxBitmapButton(m_pillPanel, ID_BTN_PLAY_PAUSE, Icons::Get(Icons::Play, btnCol, wxSize(28, 28)), wxDefaultPosition, wxSize(28, 28), wxBORDER_NONE);
    m_btnPlayPause->SetBitmapMargins(0, 0);

    m_btnNext = new wxBitmapButton(m_pillPanel, ID_BTN_NEXT, Icons::Get(Icons::Next, btnCol, wxSize(24, 24)), wxDefaultPosition, wxSize(24, 24), wxBORDER_NONE);
    m_btnNext->SetBitmapMargins(0, 0);

    leftCtrlSizer->Add(m_btnPrev, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    leftCtrlSizer->Add(m_btnPlayPause, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    leftCtrlSizer->Add(m_btnNext, 0, wxALIGN_CENTER_VERTICAL, 0);

    // Column 2: Track Info
    wxBoxSizer* centerTrackSizer = new wxBoxSizer(wxHORIZONTAL);
    wxImage dummy(40, 40);
    wxColour dummyBg = wxSystemSettings::GetColour(wxSYS_COLOUR_BTNFACE);
    dummy.SetRGB(wxRect(0, 0, 40, 40), dummyBg.Red(), dummyBg.Green(), dummyBg.Blue());
    m_coverView = new wxStaticBitmap(m_pillPanel, wxID_ANY, wxBitmap(dummy));

    wxBoxSizer* metaTextSizer = new wxBoxSizer(wxVERTICAL);
    m_lblTitle = new wxStaticText(m_pillPanel, wxID_ANY, "No track playing");
    m_lblTitle->SetFont(wxFontInfo(wxSize(0, 12)).Bold().Family(wxFONTFAMILY_DEFAULT));

    m_lblArtist = new wxStaticText(m_pillPanel, wxID_ANY, "Fluyer");
    m_lblArtist->SetForegroundColour(grayCol);
    m_lblArtist->SetFont(wxFontInfo(wxSize(0, 10)).Family(wxFONTFAMILY_DEFAULT));

    metaTextSizer->Add(m_lblTitle, 0, wxEXPAND);
    metaTextSizer->Add(m_lblArtist, 0, wxEXPAND);

    centerTrackSizer->Add(m_coverView, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 8);
    centerTrackSizer->Add(metaTextSizer, 1, wxALIGN_CENTER_VERTICAL);

    // Column 3: Secondary Controls
    wxBoxSizer* rightExtraSizer = new wxBoxSizer(wxHORIZONTAL);
    m_btnRepeat = new wxBitmapButton(m_pillPanel, ID_BTN_REPEAT, Icons::Get(Icons::Repeat, grayCol, wxSize(20, 20)), wxDefaultPosition, wxSize(20, 20), wxBORDER_NONE);
    m_btnRepeat->SetBitmapMargins(0, 0);

    m_btnShuffle = new wxBitmapButton(m_pillPanel, ID_BTN_SHUFFLE, Icons::Get(Icons::Shuffle, grayCol, wxSize(20, 20)), wxDefaultPosition, wxSize(20, 20), wxBORDER_NONE);
    m_btnShuffle->SetBitmapMargins(0, 0);
    
    m_volIcon = new wxStaticBitmap(m_pillPanel, wxID_ANY, Icons::Get(Icons::SpeakerHigh, grayCol, wxSize(18, 18)));
    m_volSlider = new wxSlider(m_pillPanel, ID_SLIDER_VOL, 100, 0, 100, wxDefaultPosition, wxSize(90, 18), wxSL_HORIZONTAL);

    rightExtraSizer->Add(m_btnRepeat, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    rightExtraSizer->Add(m_btnShuffle, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 10);
    rightExtraSizer->Add(m_volIcon, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 4);
    rightExtraSizer->Add(m_volSlider, 0, wxALIGN_CENTER_VERTICAL, 0);

    pillSizer->Add(leftCtrlSizer, 0, wxALIGN_CENTER_VERTICAL | wxLEFT, 12);
    pillSizer->AddStretchSpacer(1);
    pillSizer->Add(centerTrackSizer, 0, wxALIGN_CENTER, 8);
    pillSizer->AddStretchSpacer(1);
    pillSizer->Add(rightExtraSizer, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 12);

    m_pillPanel->SetSizer(pillSizer);
    outerSizer->Add(m_pillPanel, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);

    SetSizer(outerSizer);
    UpdateState();
}

PlayerBarCtrl::~PlayerBarCtrl() {
    if (m_progressTimer.IsRunning()) {
        m_progressTimer.Stop();
    }
}

void PlayerBarCtrl::OnSize(wxSizeEvent& evt) {
    Refresh();
    evt.Skip();
}

void PlayerBarCtrl::UpdateProgress() {
    if (!m_playerService) return;
    const auto& state = m_playerService->GetState();

    if (state.duration_ms > 0) {
        uint64_t pos = m_playerService->GetPosition();
        m_lblTimePos->SetLabel(wxString::FromUTF8(PlayerService::FormatTime(pos)));
        m_lblTimeDur->SetLabel(wxString::FromUTF8(PlayerService::FormatTime(state.duration_ms)));

        int sliderPos = static_cast<int>((static_cast<double>(pos) / state.duration_ms) * 1000.0);
        m_seekSlider->SetValue(std::clamp(sliderPos, 0, 1000));
    } else {
        m_lblTimePos->SetLabel("0:00");
        m_lblTimeDur->SetLabel("0:00");
        m_seekSlider->SetValue(0);
    }
}

void PlayerBarCtrl::OnTimer(wxTimerEvent& WXUNUSED(evt)) {
    UpdateProgress();
}

void PlayerBarCtrl::UpdateState() {
    if (!m_playerService) return;
    const auto& state = m_playerService->GetState();

    wxColour btnCol = wxSystemSettings::GetColour(wxSYS_COLOUR_BTNTEXT);
    wxColour grayCol = wxSystemSettings::GetColour(wxSYS_COLOUR_GRAYTEXT);

    m_btnPlayPause->SetBitmap(state.is_playing ? Icons::Get(Icons::Pause, btnCol, wxSize(28, 28))
                                              : Icons::Get(Icons::Play, btnCol, wxSize(28, 28)));
    m_btnPlayPause->Refresh();

    if (state.is_playing) {
        if (!m_progressTimer.IsRunning()) {
            m_progressTimer.Start(250);
        }
    } else {
        if (m_progressTimer.IsRunning()) {
            m_progressTimer.Stop();
        }
    }

    UpdateProgress();

    m_btnShuffle->SetBitmap(state.is_shuffled ? Icons::Get(Icons::Shuffle, wxColour("#1ED760"), wxSize(20, 20))
                                              : Icons::Get(Icons::Shuffle, grayCol, wxSize(20, 20)));
    m_btnShuffle->Refresh();

    if (state.repeat_mode == FluyerRepeatMode::All) {
        m_btnRepeat->SetBitmap(Icons::Get(Icons::Repeat, wxColour("#1ED760"), wxSize(20, 20)));
    } else if (state.repeat_mode == FluyerRepeatMode::One) {
        m_btnRepeat->SetBitmap(Icons::Get(Icons::RepeatOnce, wxColour("#1ED760"), wxSize(20, 20)));
    } else {
        m_btnRepeat->SetBitmap(Icons::Get(Icons::Repeat, grayCol, wxSize(20, 20)));
    }
    m_btnRepeat->Refresh();

    m_pillPanel->Layout();
}

void PlayerBarCtrl::UpdateTrack() {
    if (!m_playerService) return;
    const auto& track = m_playerService->GetCurrentTrack();

    m_lblTitle->SetLabel(wxString::FromUTF8(track.title.empty() ? "No track playing" : track.title));
    wxString sub = wxString::FromUTF8(track.artist.empty() ? "Fluyer" : track.artist);
    if (!track.album.empty()) sub += " • " + wxString::FromUTF8(track.album);
    m_lblArtist->SetLabel(sub);

    OnCoverLoaded();
    m_pillPanel->Layout();
}

void PlayerBarCtrl::OnCoverLoaded() {
    if (m_imageService) {
        wxBitmap cover = m_imageService->GetCurrentImage(40, GetContentScaleFactor());
        if (cover.IsOk()) {
            m_coverView->SetBitmap(cover);
        }
    }
}

void PlayerBarCtrl::OnPlayPause(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->TogglePlay();
}

void PlayerBarCtrl::OnNext(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->Next();
}

void PlayerBarCtrl::OnPrev(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->Previous();
}

void PlayerBarCtrl::OnShuffle(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->Shuffle();
}

void PlayerBarCtrl::OnRepeat(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->CycleRepeat();
}

void PlayerBarCtrl::OnSeek(wxCommandEvent& WXUNUSED(evt)) {
    if (!m_playerService) return;
    int val = m_seekSlider->GetValue();
    float pct = static_cast<float>(val) / 1000.0f;
    const auto& state = m_playerService->GetState();
    if (state.duration_ms > 0) {
        uint64_t targetPos = static_cast<uint64_t>(pct * state.duration_ms);
        m_lblTimePos->SetLabel(wxString::FromUTF8(PlayerService::FormatTime(targetPos)));
    }
    m_playerService->SeekPercent(pct);
}

void PlayerBarCtrl::OnVolume(wxCommandEvent& WXUNUSED(evt)) {
    if (!m_playerService) return;
    int val = m_volSlider->GetValue();
    m_playerService->SetVolume(static_cast<float>(val) / 100.0f);
    if (m_volIcon) {
        wxColour grayCol = wxSystemSettings::GetColour(wxSYS_COLOUR_GRAYTEXT);
        m_volIcon->SetBitmap(val == 0 ? Icons::Get(Icons::SpeakerMute, grayCol, wxSize(18, 18))
                                      : Icons::Get(Icons::SpeakerHigh, grayCol, wxSize(18, 18)));
    }
}

void PlayerBarCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);
    dc.SetBackground(wxBrush(GetBackgroundColour()));
    dc.Clear();
}
