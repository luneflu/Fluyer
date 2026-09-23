#include "PlayerBarCtrl.h"
#include "common/Icons.h"
#include <wx/dcbuffer.h>
#include <wx/settings.h>
#include <algorithm>
#include <cmath>

#if defined(__WXOSX__) || defined(__APPLE__)
#import <AppKit/AppKit.h>
#endif

// ponytail: fixes wxWidgets macOS bug where SetBitmap resets NSButton imagePosition to NSImageLeft (x=0)
static void SetButtonBitmap(wxButton* btn, const wxBitmapBundle& bmp) {
    if (!btn) return;
    btn->SetBitmap(bmp);
#if defined(__WXOSX__) || defined(__APPLE__)
    NSButton* nsBtn = (NSButton*)btn->GetHandle();
    if (nsBtn) {
        [nsBtn setImagePosition:NSImageOnly];
    }
#endif
}

enum {
    ID_BTN_PLAY_PAUSE = 2001,
    ID_BTN_PREV,
    ID_BTN_NEXT,
    ID_BTN_SHUFFLE,
    ID_BTN_REPEAT,
    ID_BTN_VOL,
    ID_TIMER_PROGRESS
};

const wxSize kCtrlBtnSize(32, 32);
const wxSize kSecondaryBtnSize(24, 24);

// ponytail: custom lightweight progress bar matching ProgressBar.svelte
class ProgressBarCtrl : public wxPanel {
public:
    ProgressBarCtrl(wxWindow* parent, wxWindowID id = wxID_ANY, int barHeight = 5,
                    bool showTooltip = false, const wxSize& size = wxDefaultSize)
        : wxPanel(parent, id, wxDefaultPosition, size, wxNO_BORDER),
          m_barHeight(barHeight),
          m_showTooltip(showTooltip) {
        SetBackgroundStyle(wxBG_STYLE_PAINT);
        SetCursor(wxCursor(wxCURSOR_HAND));
    }

    void SetPercentage(float pct) {
        float clamped = std::clamp(pct, 0.0f, 1.0f);
        if (std::abs(clamped - m_percentage) > 0.0005f) {
            m_percentage = clamped;
            Refresh();
        }
    }

    float GetPercentage() const { return m_percentage; }

    void SetOnSeek(std::function<void(float)> cb) { m_onSeek = std::move(cb); }
    void SetTooltipFormatter(std::function<std::string(float)> fmt) { m_formatter = std::move(fmt); }

private:
    void OnPaint(wxPaintEvent& WXUNUSED(evt)) {
        wxAutoBufferedPaintDC dc(this);
        dc.SetBackground(wxBrush(GetParent() ? GetParent()->GetBackgroundColour() : GetBackgroundColour()));
        dc.Clear();

        wxRect client = GetClientRect();
        if (client.width <= 0 || client.height <= 0) return;

        int trackY = (client.height - m_barHeight) / 2;
        int trackW = client.width;
        double radius = m_barHeight / 2.0;

        bool isDark = wxSystemSettings::GetAppearance().IsDark();
        wxColour trackBg = isDark ? wxColour(255, 255, 255, 65) : wxColour(0, 0, 0, 45);
        wxColour trackFill = isDark ? wxColour(255, 255, 255, 240) : wxColour(0, 0, 0, 210);

        dc.SetBrush(wxBrush(trackBg));
        dc.SetPen(*wxTRANSPARENT_PEN);
        dc.DrawRoundedRectangle(0, trackY, trackW, m_barHeight, radius);

        int fillW = static_cast<int>(trackW * m_percentage);
        if (fillW > 0) {
            dc.SetBrush(wxBrush(trackFill));
            dc.DrawRoundedRectangle(0, trackY, fillW, m_barHeight, radius);
        }
    }

    void OnLeftDown(wxMouseEvent& evt) {
        m_isDragging = true;
        if (!HasCapture()) CaptureMouse();
        UpdateFromMouse(evt);
    }

    void OnLeftUp(wxMouseEvent& evt) {
        if (m_isDragging) {
            m_isDragging = false;
            if (HasCapture()) ReleaseMouse();
            UpdateFromMouse(evt);
        }
    }

    void OnMotion(wxMouseEvent& evt) {
        if (m_isDragging) {
            UpdateFromMouse(evt);
        } else if (m_showTooltip && m_formatter) {
            float pct = GetMousePct(evt);
            SetToolTip(wxString::FromUTF8(m_formatter(pct)));
        }
    }

    float GetMousePct(const wxMouseEvent& evt) const {
        int w = GetClientSize().GetWidth();
        if (w <= 0) return 0.0f;
        return std::clamp(static_cast<float>(evt.GetX()) / static_cast<float>(w), 0.0f, 1.0f);
    }

    void UpdateFromMouse(const wxMouseEvent& evt) {
        float pct = GetMousePct(evt);
        m_percentage = pct;
        Refresh();
        if (m_onSeek) {
            m_onSeek(pct);
        }
    }

    int m_barHeight = 5;
    bool m_showTooltip = false;
    float m_percentage = 0.0f;
    bool m_isDragging = false;
    std::function<void(float)> m_onSeek;
    std::function<std::string(float)> m_formatter;

    wxDECLARE_EVENT_TABLE();
};

wxBEGIN_EVENT_TABLE(ProgressBarCtrl, wxPanel)
    EVT_PAINT(ProgressBarCtrl::OnPaint)
    EVT_LEFT_DOWN(ProgressBarCtrl::OnLeftDown)
    EVT_LEFT_UP(ProgressBarCtrl::OnLeftUp)
    EVT_MOTION(ProgressBarCtrl::OnMotion)
wxEND_EVENT_TABLE()

// ponytail: rounded-full glass capsule matching View.svelte rounded-full
class PillPanel : public wxPanel {
public:
    PillPanel(wxWindow* parent, wxWindowID id = wxID_ANY)
        : wxPanel(parent, id, wxDefaultPosition, wxDefaultSize, wxNO_BORDER) {
        SetBackgroundStyle(wxBG_STYLE_PAINT);
    }

private:
    void OnPaint(wxPaintEvent& WXUNUSED(evt)) {
        wxAutoBufferedPaintDC dc(this);
        dc.SetBackground(wxBrush(GetParent() ? GetParent()->GetBackgroundColour() : GetBackgroundColour()));
        dc.Clear();

        wxRect rect = GetClientRect();
        rect.Deflate(1, 1);
        if (rect.width <= 0 || rect.height <= 0) return;

        bool isDark = wxSystemSettings::GetAppearance().IsDark();
        wxColour fillCol = isDark ? wxColour(255, 255, 255, 22) : wxColour(0, 0, 0, 14);
        wxColour borderCol = isDark ? wxColour(255, 255, 255, 55) : wxColour(0, 0, 0, 38);

        dc.SetBrush(wxBrush(fillCol));
        dc.SetPen(wxPen(borderCol, 1));
        dc.DrawRoundedRectangle(rect, rect.height / 2.0);
    }

    wxDECLARE_EVENT_TABLE();
};

wxBEGIN_EVENT_TABLE(PillPanel, wxPanel)
    EVT_PAINT(PillPanel::OnPaint)
wxEND_EVENT_TABLE()

wxBEGIN_EVENT_TABLE(PlayerBarCtrl, wxPanel)
    EVT_PAINT(PlayerBarCtrl::OnPaint)
    EVT_SIZE(PlayerBarCtrl::OnSize)
    EVT_BUTTON(ID_BTN_PLAY_PAUSE, PlayerBarCtrl::OnPlayPause)
    EVT_BUTTON(ID_BTN_PREV, PlayerBarCtrl::OnPrev)
    EVT_BUTTON(ID_BTN_NEXT, PlayerBarCtrl::OnNext)
    EVT_BUTTON(ID_BTN_SHUFFLE, PlayerBarCtrl::OnShuffle)
    EVT_BUTTON(ID_BTN_REPEAT, PlayerBarCtrl::OnRepeat)
    EVT_BUTTON(ID_BTN_VOL, PlayerBarCtrl::OnVolumeBtn)
    EVT_TIMER(ID_TIMER_PROGRESS, PlayerBarCtrl::OnTimer)
wxEND_EVENT_TABLE()

PlayerBarCtrl::PlayerBarCtrl(wxWindow* parent, PlayerService* playerService, ImageService* imageService, wxWindowID id)
    : wxPanel(parent, id, wxDefaultPosition, wxSize(-1, 104), wxNO_BORDER),
      m_playerService(playerService),
      m_imageService(imageService),
      m_progressTimer(this, ID_TIMER_PROGRESS) {
    SetBackgroundStyle(wxBG_STYLE_PAINT);

    wxBoxSizer* outerSizer = new wxBoxSizer(wxVERTICAL);

    wxColour activeCol(255, 255, 255, 255);
    wxColour inactiveCol(255, 255, 255, 128);

    // 1. Sleek top progress bar (full width, 5px height, hover tooltip)
    m_progressBar = new ProgressBarCtrl(this, wxID_ANY, 5, true, wxSize(-1, 10));
    m_progressBar->SetTooltipFormatter([this](float pct) -> std::string {
        if (!m_playerService) return "0:00";
        const auto& state = m_playerService->GetState();
        uint64_t targetPos = static_cast<uint64_t>(pct * state.duration_ms);
        return PlayerService::FormatTime(targetPos);
    });
    m_progressBar->SetOnSeek([this](float pct) {
        if (m_playerService) {
            m_playerService->SeekPercent(pct);
        }
    });

    outerSizer->Add(m_progressBar, 0, wxEXPAND | wxLEFT | wxRIGHT | wxTOP, 12);
    outerSizer->AddSpacer(6);

    // 2. Control Pill Container (rounded-full glass capsule)
    m_pillPanel = new PillPanel(this, wxID_ANY);

    wxBoxSizer* pillSizer = new wxBoxSizer(wxHORIZONTAL);

    // Column 1: Playback Controls (Previous, Play/Pause, Next)
    wxBoxSizer* leftCtrlSizer = new wxBoxSizer(wxHORIZONTAL);
    m_btnPrev = new wxBitmapButton(m_pillPanel, ID_BTN_PREV, wxBitmapBundle(), wxDefaultPosition, kCtrlBtnSize, wxBORDER_NONE);
    m_btnPrev->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnPrev, Icons::Get(Icons::Prev, activeCol, kCtrlBtnSize));

    m_btnPlayPause = new wxBitmapButton(m_pillPanel, ID_BTN_PLAY_PAUSE, wxBitmapBundle(), wxDefaultPosition, kCtrlBtnSize, wxBORDER_NONE);
    m_btnPlayPause->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnPlayPause, Icons::Get(Icons::Play, activeCol, kCtrlBtnSize));

    m_btnNext = new wxBitmapButton(m_pillPanel, ID_BTN_NEXT, wxBitmapBundle(), wxDefaultPosition, kCtrlBtnSize, wxBORDER_NONE);
    m_btnNext->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnNext, Icons::Get(Icons::Next, activeCol, kCtrlBtnSize));

    leftCtrlSizer->Add(m_btnPrev, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    leftCtrlSizer->Add(m_btnPlayPause, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    leftCtrlSizer->Add(m_btnNext, 0, wxALIGN_CENTER_VERTICAL, 0);

    // Column 2: Track Info (Centered: Cover + Title + Artist)
    wxBoxSizer* centerTrackSizer = new wxBoxSizer(wxHORIZONTAL);
    wxImage dummy(40, 40);
    wxColour dummyBg = wxSystemSettings::GetColour(wxSYS_COLOUR_BTNFACE);
    dummy.SetRGB(wxRect(0, 0, 40, 40), dummyBg.Red(), dummyBg.Green(), dummyBg.Blue());
    m_coverView = new wxStaticBitmap(m_pillPanel, wxID_ANY, wxBitmap(dummy), wxDefaultPosition, wxSize(40, 40));

    wxBoxSizer* metaTextSizer = new wxBoxSizer(wxVERTICAL);
    m_lblTitle = new wxStaticText(m_pillPanel, wxID_ANY, "No track playing", wxDefaultPosition, wxDefaultSize, wxST_ELLIPSIZE_END);
    m_lblTitle->SetFont(wxFontInfo(wxSize(0, 14)).Weight(wxFONTWEIGHT_MEDIUM).Family(wxFONTFAMILY_DEFAULT));

    m_lblArtist = new wxStaticText(m_pillPanel, wxID_ANY, "Fluyer", wxDefaultPosition, wxDefaultSize, wxST_ELLIPSIZE_END);
    m_lblArtist->SetForegroundColour(inactiveCol);
    m_lblArtist->SetFont(wxFontInfo(wxSize(0, 12)).Family(wxFONTFAMILY_DEFAULT));

    metaTextSizer->Add(m_lblTitle, 0, wxEXPAND);
    metaTextSizer->AddSpacer(2);
    metaTextSizer->Add(m_lblArtist, 0, wxEXPAND);

    centerTrackSizer->Add(m_coverView, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 10);
    centerTrackSizer->Add(metaTextSizer, 1, wxALIGN_CENTER_VERTICAL);

    // Column 3: Secondary Controls (Repeat, Shuffle, Volume button, Volume bar)
    wxBoxSizer* rightExtraSizer = new wxBoxSizer(wxHORIZONTAL);
    m_btnRepeat = new wxBitmapButton(m_pillPanel, ID_BTN_REPEAT, Icons::Get(Icons::Repeat, inactiveCol, kSecondaryBtnSize), wxDefaultPosition, kSecondaryBtnSize, wxBORDER_NONE);
    m_btnRepeat->SetBitmapMargins(0, 0);

    m_btnShuffle = new wxBitmapButton(m_pillPanel, ID_BTN_SHUFFLE, Icons::Get(Icons::Shuffle, inactiveCol, kSecondaryBtnSize), wxDefaultPosition, kSecondaryBtnSize, wxBORDER_NONE);
    m_btnShuffle->SetBitmapMargins(0, 0);

    m_btnVol = new wxBitmapButton(m_pillPanel, ID_BTN_VOL, Icons::Get(Icons::SpeakerHigh, activeCol, kSecondaryBtnSize), wxDefaultPosition, kSecondaryBtnSize, wxBORDER_NONE);
    m_btnVol->SetBitmapMargins(0, 0);
    m_btnVol->SetToolTip("Volume");

    m_volBar = new ProgressBarCtrl(m_pillPanel, wxID_ANY, 4, false, wxSize(96, 12));
    m_volBar->SetPercentage(1.0f);
    m_volBar->SetOnSeek([this](float pct) {
        if (m_playerService) {
            m_playerService->SetVolume(pct);
            UpdateVolumeIcon();
        }
    });

    rightExtraSizer->Add(m_btnRepeat, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 8);
    rightExtraSizer->Add(m_btnShuffle, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 10);
    rightExtraSizer->Add(m_btnVol, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    rightExtraSizer->Add(m_volBar, 0, wxALIGN_CENTER_VERTICAL, 0);

    pillSizer->Add(leftCtrlSizer, 0, wxALIGN_CENTER_VERTICAL | wxLEFT, 18);
    pillSizer->AddStretchSpacer(1);
    pillSizer->Add(centerTrackSizer, 0, wxALIGN_CENTER, 8);
    pillSizer->AddStretchSpacer(1);
    pillSizer->Add(rightExtraSizer, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 18);

    m_pillPanel->SetSizer(pillSizer);
    outerSizer->Add(m_pillPanel, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 12);

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
    if (!m_playerService || !m_progressBar) return;
    const auto& state = m_playerService->GetState();

    if (state.duration_ms > 0) {
        uint64_t pos = m_playerService->GetPosition();
        float pct = static_cast<float>(pos) / static_cast<float>(state.duration_ms);
        m_progressBar->SetPercentage(pct);
    } else {
        m_progressBar->SetPercentage(0.0f);
    }
}

void PlayerBarCtrl::OnTimer(wxTimerEvent& WXUNUSED(evt)) {
    UpdateProgress();
}

void PlayerBarCtrl::UpdateVolumeIcon() {
    float vol = m_volBar ? m_volBar->GetPercentage() : 1.0f;
    wxColour activeCol(255, 255, 255, 255);
    wxColour inactiveCol(255, 255, 255, 128);
    if (m_btnVol) {
        SetButtonBitmap(m_btnVol, vol <= 0.001f ? Icons::Get(Icons::SpeakerMute, inactiveCol, kSecondaryBtnSize)
                                                : Icons::Get(Icons::SpeakerHigh, activeCol, kSecondaryBtnSize));
        m_btnVol->Refresh();
    }
}

void PlayerBarCtrl::UpdateState() {
    if (!m_playerService) return;
    const auto& state = m_playerService->GetState();

    wxColour activeCol(255, 255, 255, 255);
    wxColour inactiveCol(255, 255, 255, 128);

    SetButtonBitmap(m_btnPlayPause, state.is_playing ? Icons::Get(Icons::Pause, activeCol, kCtrlBtnSize)
                                                    : Icons::Get(Icons::Play, activeCol, kCtrlBtnSize));
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

    SetButtonBitmap(m_btnShuffle, state.is_shuffled ? Icons::Get(Icons::Shuffle, activeCol, kSecondaryBtnSize)
                                                    : Icons::Get(Icons::Shuffle, inactiveCol, kSecondaryBtnSize));
    m_btnShuffle->Refresh();

    if (state.repeat_mode == FluyerRepeatMode::All) {
        SetButtonBitmap(m_btnRepeat, Icons::Get(Icons::Repeat, activeCol, kSecondaryBtnSize));
    } else if (state.repeat_mode == FluyerRepeatMode::One) {
        SetButtonBitmap(m_btnRepeat, Icons::Get(Icons::RepeatOnce, activeCol, kSecondaryBtnSize));
    } else {
        SetButtonBitmap(m_btnRepeat, Icons::Get(Icons::Repeat, inactiveCol, kSecondaryBtnSize));
    }
    m_btnRepeat->Refresh();

    UpdateVolumeIcon();

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

void PlayerBarCtrl::OnVolumeBtn(wxCommandEvent& WXUNUSED(evt)) {
    if (!m_playerService || !m_volBar) return;
    float currentVol = m_volBar->GetPercentage();
    if (currentVol > 0.001f) {
        m_previousVolume = currentVol;
        m_playerService->SetVolume(0.0f);
        m_volBar->SetPercentage(0.0f);
    } else {
        float restoreVol = m_previousVolume > 0.05f ? m_previousVolume : 1.0f;
        m_playerService->SetVolume(restoreVol);
        m_volBar->SetPercentage(restoreVol);
    }
    UpdateVolumeIcon();
}

void PlayerBarCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxAutoBufferedPaintDC dc(this);
    dc.SetBackground(wxBrush(GetParent() ? GetParent()->GetBackgroundColour() : GetBackgroundColour()));
    dc.Clear();
}
