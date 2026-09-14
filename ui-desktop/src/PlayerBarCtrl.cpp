#include "PlayerBarCtrl.h"
#include "Icons.h"
#include <wx/dcbuffer.h>
#include <wx/bmpbndl.h>
#include <wx/statbmp.h>
#include <wx/bmpbuttn.h>
#include <iomanip>
#include <sstream>

using Icons::CreateSVGIcon;

enum {
    ID_BTN_PLAY_PAUSE = 2001,
    ID_BTN_PREV,
    ID_BTN_NEXT,
    ID_BTN_SHUFFLE,
    ID_BTN_REPEAT,
    ID_SLIDER_SEEK,
    ID_SLIDER_VOL
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
wxEND_EVENT_TABLE()

static wxString FormatTime(uint64_t ms) {
    uint64_t totalSec = ms / 1000;
    uint64_t min = totalSec / 60;
    uint64_t sec = totalSec % 60;
    std::ostringstream ss;
    ss << min << ":" << std::setw(2) << std::setfill('0') << sec;
    return wxString(ss.str());
}

PlayerBarCtrl::PlayerBarCtrl(wxWindow* parent, wxWindowID id)
    : wxPanel(parent, id, wxDefaultPosition, wxSize(-1, 108), wxNO_BORDER) {
    SetBackgroundStyle(wxBG_STYLE_PAINT);
    SetBackgroundColour(wxColour(14, 14, 14));

    wxBoxSizer* outerSizer = new wxBoxSizer(wxVERTICAL);

    // 1. Top progress bar (matching Svelte ProgressBar)
    wxBoxSizer* progSizer = new wxBoxSizer(wxHORIZONTAL);
    m_lblTimePos = new wxStaticText(this, wxID_ANY, "0:00", wxDefaultPosition, wxSize(42, -1), wxALIGN_RIGHT);
    m_lblTimePos->SetForegroundColour(wxColour(140, 140, 140));
    m_lblTimePos->SetFont(wxFontInfo(wxSize(0, 10)).Family(wxFONTFAMILY_DEFAULT));

    m_seekSlider = new wxSlider(this, ID_SLIDER_SEEK, 0, 0, 1000, wxDefaultPosition, wxDefaultSize, wxSL_HORIZONTAL);

    m_lblTimeDur = new wxStaticText(this, wxID_ANY, "0:00", wxDefaultPosition, wxSize(42, -1), wxALIGN_LEFT);
    m_lblTimeDur->SetForegroundColour(wxColour(140, 140, 140));
    m_lblTimeDur->SetFont(wxFontInfo(wxSize(0, 10)).Family(wxFONTFAMILY_DEFAULT));

    progSizer->Add(m_lblTimePos, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    progSizer->Add(m_seekSlider, 1, wxALIGN_CENTER_VERTICAL);
    progSizer->Add(m_lblTimeDur, 0, wxALIGN_CENTER_VERTICAL | wxLEFT, 6);

    outerSizer->Add(progSizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxTOP, 10);
    outerSizer->AddSpacer(4);

    // 2. Rounded Pill Container (matching Svelte <View class="rounded-full px-3 py-2"> 3-column grid)
    m_pillPanel = new wxPanel(this, wxID_ANY);
    m_pillPanel->SetBackgroundColour(wxColour(24, 24, 24));

    wxBoxSizer* pillSizer = new wxBoxSizer(wxHORIZONTAL);

    // Column 1: Playback Controls (Previous, Play/Pause, Next)
    wxBoxSizer* leftCtrlSizer = new wxBoxSizer(wxHORIZONTAL);
    m_btnPrev = new wxBitmapButton(m_pillPanel, ID_BTN_PREV, CreateSVGIcon(Icons::SKIP_BACK_CIRCLE, "#D0D0D0", wxSize(24, 24)), wxDefaultPosition, wxSize(36, 30), wxBORDER_NONE);
    m_btnPrev->SetBackgroundColour(wxColour(24, 24, 24));

    m_btnPlayPause = new wxBitmapButton(m_pillPanel, ID_BTN_PLAY_PAUSE, CreateSVGIcon(Icons::PLAY_CIRCLE, "#FFFFFF", wxSize(28, 28)), wxDefaultPosition, wxSize(46, 32), wxBORDER_NONE);
    m_btnPlayPause->SetBackgroundColour(wxColour(24, 24, 24));

    m_btnNext = new wxBitmapButton(m_pillPanel, ID_BTN_NEXT, CreateSVGIcon(Icons::SKIP_FORWARD_CIRCLE, "#D0D0D0", wxSize(24, 24)), wxDefaultPosition, wxSize(36, 30), wxBORDER_NONE);
    m_btnNext->SetBackgroundColour(wxColour(24, 24, 24));

    leftCtrlSizer->Add(m_btnPrev, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 4);
    leftCtrlSizer->Add(m_btnPlayPause, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 4);
    leftCtrlSizer->Add(m_btnNext, 0, wxALIGN_CENTER_VERTICAL, 0);

    // Column 2: Center Track Info (Cover + Title/Artist)
    wxBoxSizer* centerTrackSizer = new wxBoxSizer(wxHORIZONTAL);
    
    // Placeholder 40x40 cover
    wxImage dummy(40, 40);
    dummy.SetRGB(wxRect(0, 0, 40, 40), 40, 40, 40);
    m_coverView = new wxStaticBitmap(m_pillPanel, wxID_ANY, wxBitmap(dummy));

    wxBoxSizer* metaTextSizer = new wxBoxSizer(wxVERTICAL);
    m_lblTitle = new wxStaticText(m_pillPanel, wxID_ANY, "No track playing");
    m_lblTitle->SetForegroundColour(wxColour(240, 240, 240));
    m_lblTitle->SetFont(wxFontInfo(wxSize(0, 12)).Bold().Family(wxFONTFAMILY_DEFAULT));

    m_lblArtist = new wxStaticText(m_pillPanel, wxID_ANY, "Fluyer");
    m_lblArtist->SetForegroundColour(wxColour(160, 160, 160));
    m_lblArtist->SetFont(wxFontInfo(wxSize(0, 10)).Family(wxFONTFAMILY_DEFAULT));

    metaTextSizer->Add(m_lblTitle, 0, wxEXPAND);
    metaTextSizer->Add(m_lblArtist, 0, wxEXPAND);

    centerTrackSizer->Add(m_coverView, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 8);
    centerTrackSizer->Add(metaTextSizer, 1, wxALIGN_CENTER_VERTICAL);

    // Column 3: Right Controls (Repeat, Shuffle, Volume Icon, Volume Slider)
    wxBoxSizer* rightExtraSizer = new wxBoxSizer(wxHORIZONTAL);
    m_btnRepeat = new wxBitmapButton(m_pillPanel, ID_BTN_REPEAT, CreateSVGIcon(Icons::REPEAT, "#8C8C8C", wxSize(20, 20)), wxDefaultPosition, wxSize(34, 28), wxBORDER_NONE);
    m_btnRepeat->SetBackgroundColour(wxColour(24, 24, 24));

    m_btnShuffle = new wxBitmapButton(m_pillPanel, ID_BTN_SHUFFLE, CreateSVGIcon(Icons::SHUFFLE, "#8C8C8C", wxSize(20, 20)), wxDefaultPosition, wxSize(34, 28), wxBORDER_NONE);
    m_btnShuffle->SetBackgroundColour(wxColour(24, 24, 24));
    
    m_volIcon = new wxStaticBitmap(m_pillPanel, wxID_ANY, CreateSVGIcon(Icons::SPEAKER_HIGH, "#A0A0A0", wxSize(18, 18)));
    m_volSlider = new wxSlider(m_pillPanel, ID_SLIDER_VOL, 100, 0, 100, wxDefaultPosition, wxSize(90, 18), wxSL_HORIZONTAL);

    rightExtraSizer->Add(m_btnRepeat, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    rightExtraSizer->Add(m_btnShuffle, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 10);
    rightExtraSizer->Add(m_volIcon, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 4);
    rightExtraSizer->Add(m_volSlider, 0, wxALIGN_CENTER_VERTICAL, 0);

    pillSizer->Add(leftCtrlSizer, 1, wxALIGN_CENTER_VERTICAL | wxLEFT, 12);
    pillSizer->Add(centerTrackSizer, 2, wxALIGN_CENTER_VERTICAL | wxLEFT | wxRIGHT, 8);
    pillSizer->Add(rightExtraSizer, 1, wxALIGN_RIGHT | wxALIGN_CENTER_VERTICAL | wxRIGHT, 12);

    m_pillPanel->SetSizer(pillSizer);

    outerSizer->Add(m_pillPanel, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 8);

    SetSizer(outerSizer);
}

void PlayerBarCtrl::OnSize(wxSizeEvent& evt) {
    Refresh();
    evt.Skip();
}

void PlayerBarCtrl::SetEngine(FluyerEngine* engine) {
    m_engine = engine;
}

void PlayerBarCtrl::UpdateState(const FluyerPlayerState& state) {
    m_state = state;
    m_btnPlayPause->SetBitmap(state.is_playing ? CreateSVGIcon(Icons::PAUSE_CIRCLE, "#FFFFFF", wxSize(28, 28))
                                              : CreateSVGIcon(Icons::PLAY_CIRCLE, "#FFFFFF", wxSize(28, 28)));
    
    if (state.duration_ms > 0) {
        m_lblTimePos->SetLabel(FormatTime(state.position_ms));
        m_lblTimeDur->SetLabel(FormatTime(state.duration_ms));

        int pos = static_cast<int>((static_cast<double>(state.position_ms) / state.duration_ms) * 1000.0);
        m_seekSlider->SetValue(std::clamp(pos, 0, 1000));
    } else {
        m_lblTimePos->SetLabel("0:00");
        m_lblTimeDur->SetLabel("0:00");
        m_seekSlider->SetValue(0);
    }

    if (state.is_shuffled) {
        m_btnShuffle->SetBitmap(CreateSVGIcon(Icons::SHUFFLE, "#1ED760", wxSize(20, 20)));
    } else {
        m_btnShuffle->SetBitmap(CreateSVGIcon(Icons::SHUFFLE, "#8C8C8C", wxSize(20, 20)));
    }

    if (state.repeat_mode == FluyerRepeatMode::All) {
        m_btnRepeat->SetBitmap(CreateSVGIcon(Icons::REPEAT, "#1ED760", wxSize(20, 20)));
    } else if (state.repeat_mode == FluyerRepeatMode::One) {
        m_btnRepeat->SetBitmap(CreateSVGIcon(Icons::REPEAT_ONCE, "#1ED760", wxSize(20, 20)));
    } else {
        m_btnRepeat->SetBitmap(CreateSVGIcon(Icons::REPEAT, "#8C8C8C", wxSize(20, 20)));
    }

    m_pillPanel->Layout();
}

void PlayerBarCtrl::UpdateTrack(const wxString& title, const wxString& artist, const wxString& album, uintptr_t trackIdx) {
    m_title = title.empty() ? "No track playing" : title;
    m_artist = artist.empty() ? "Fluyer" : artist;
    m_album = album;
    m_currentTrackIdx = trackIdx;

    m_lblTitle->SetLabel(m_title);
    wxString sub = m_artist;
    if (!m_album.empty()) sub += " • " + m_album;
    m_lblArtist->SetLabel(sub);

    if (m_engine) {
        uintptr_t imgLen = 0;
        uint8_t* bytes = fluyer_player_get_current_image(m_engine, &imgLen);
        if (bytes && imgLen > 0) {
            wxMemoryInputStream stream(bytes, imgLen);
            wxImage img(stream);
            if (img.IsOk()) {
                double scaleFactor = GetContentScaleFactor();
                int target = static_cast<int>(40 * scaleFactor);
                wxImage scaled = img.Scale(target, target, wxIMAGE_QUALITY_HIGH);
                m_coverThumb = wxBitmap(scaled, -1, scaleFactor);
                m_coverView->SetBitmap(m_coverThumb);
            }
            fluyer_bytes_free(bytes, imgLen);
        }
    }

    m_pillPanel->Layout();
}

void PlayerBarCtrl::OnCoverLoaded() {
    if (m_engine) {
        uintptr_t imgLen = 0;
        uint8_t* bytes = fluyer_player_get_current_image(m_engine, &imgLen);
        if (bytes && imgLen > 0) {
            wxMemoryInputStream stream(bytes, imgLen);
            wxImage img(stream);
            if (img.IsOk()) {
                double scaleFactor = GetContentScaleFactor();
                int target = static_cast<int>(40 * scaleFactor);
                wxImage scaled = img.Scale(target, target, wxIMAGE_QUALITY_HIGH);
                m_coverThumb = wxBitmap(scaled, -1, scaleFactor);
                m_coverView->SetBitmap(m_coverThumb);
            }
            fluyer_bytes_free(bytes, imgLen);
        }
    }
}

void PlayerBarCtrl::OnPlayPause(wxCommandEvent& WXUNUSED(evt)) {
    if (m_engine) fluyer_player_toggle_play(m_engine);
}

void PlayerBarCtrl::OnNext(wxCommandEvent& WXUNUSED(evt)) {
    if (m_engine) fluyer_player_next(m_engine);
}

void PlayerBarCtrl::OnPrev(wxCommandEvent& WXUNUSED(evt)) {
    if (m_engine) fluyer_player_previous(m_engine);
}

void PlayerBarCtrl::OnShuffle(wxCommandEvent& WXUNUSED(evt)) {
    if (m_engine) fluyer_player_shuffle(m_engine);
}

void PlayerBarCtrl::OnRepeat(wxCommandEvent& WXUNUSED(evt)) {
    if (!m_engine) return;
    FluyerRepeatMode nextMode;
    if (m_state.repeat_mode == FluyerRepeatMode::None) {
        nextMode = FluyerRepeatMode::All;
    } else if (m_state.repeat_mode == FluyerRepeatMode::All) {
        nextMode = FluyerRepeatMode::One;
    } else {
        nextMode = FluyerRepeatMode::None;
    }
    fluyer_player_set_repeat(m_engine, nextMode);
}

void PlayerBarCtrl::OnSeek(wxCommandEvent& WXUNUSED(evt)) {
    if (!m_engine || m_state.duration_ms == 0) return;
    int val = m_seekSlider->GetValue();
    uint64_t target_ms = static_cast<uint64_t>((static_cast<double>(val) / 1000.0) * m_state.duration_ms);
    fluyer_player_seek(m_engine, target_ms);
}

void PlayerBarCtrl::OnVolume(wxCommandEvent& WXUNUSED(evt)) {
    if (!m_engine) return;
    int val = m_volSlider->GetValue();
    float vol = static_cast<float>(val) / 100.0f;
    fluyer_player_set_volume(m_engine, vol);
    if (m_volIcon) {
        m_volIcon->SetBitmap(val == 0 ? CreateSVGIcon(Icons::SPEAKER_X, "#A0A0A0", wxSize(18, 18))
                                      : CreateSVGIcon(Icons::SPEAKER_HIGH, "#A0A0A0", wxSize(18, 18)));
    }
}

void PlayerBarCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);
    dc.SetBackground(wxBrush(wxColour(14, 14, 14)));
    dc.Clear();
}
