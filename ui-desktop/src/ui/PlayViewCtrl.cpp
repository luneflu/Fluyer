#include "PlayViewCtrl.h"
#include "ProgressBarCtrl.h"
#include "common/Icons.h"
#include <wx/dcbuffer.h>
#include <wx/graphics.h>
#include <wx/settings.h>
#include <algorithm>
#include <cmath>

#if defined(__WXOSX__) || defined(__APPLE__)
#import <AppKit/AppKit.h>
#endif

// ponytail: fixes wxWidgets macOS bug where SetBitmap resets NSButton imagePosition to NSImageLeft
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
    ID_PLAYVIEW_PLAY_PAUSE = 3001,
    ID_PLAYVIEW_PREV,
    ID_PLAYVIEW_NEXT,
    ID_PLAYVIEW_SHUFFLE,
    ID_PLAYVIEW_REPEAT,
    ID_PLAYVIEW_BACK,
    ID_PLAYVIEW_SYNC_TIMER
};

const wxSize kPlayViewBtnSize(36, 36);
const wxSize kPlayViewSmallBtnSize(26, 26);

// ponytail: Cover art display with rounded corners
class CoverPanel : public wxPanel {
public:
    CoverPanel(wxWindow* parent, wxWindowID id = wxID_ANY, const wxSize& size = wxSize(300, 300))
        : wxPanel(parent, id, wxDefaultPosition, size, wxNO_BORDER) {
        SetBackgroundStyle(wxBG_STYLE_TRANSPARENT);
        SetMinSize(size);
        SetMaxSize(size);
    }

    void SetCover(const wxBitmap& bmp) {
        m_bitmap = bmp;
        Refresh();
    }

private:
    void OnPaint(wxPaintEvent& WXUNUSED(evt)) {
        wxPaintDC dc(this);
        wxRect rect = GetClientRect();
        if (rect.width <= 0 || rect.height <= 0) return;

        std::unique_ptr<wxGraphicsContext> gc(wxGraphicsContext::Create(dc));
        if (!gc) return;

        if (m_bitmap.IsOk()) {
            gc->DrawBitmap(m_bitmap, rect.x, rect.y, rect.width, rect.height);
        } else {
            gc->SetBrush(wxBrush(wxColour(40, 40, 48)));
            gc->SetPen(*wxTRANSPARENT_PEN);
            gc->DrawRectangle(rect.x, rect.y, rect.width, rect.height);
        }

        // Subtle border
        gc->SetPen(wxPen(wxColour(255, 255, 255, 35), 1));
        gc->SetBrush(*wxTRANSPARENT_BRUSH);
        gc->DrawRectangle(rect.x, rect.y, rect.width, rect.height);
    }

    wxBitmap m_bitmap;
    wxDECLARE_EVENT_TABLE();
};

wxBEGIN_EVENT_TABLE(CoverPanel, wxPanel)
    EVT_PAINT(CoverPanel::OnPaint)
wxEND_EVENT_TABLE()

// ponytail: glass container matching View.svelte in /play
class ControlCardPanel : public wxPanel {
public:
    ControlCardPanel(wxWindow* parent, wxWindowID id = wxID_ANY)
        : wxPanel(parent, id, wxDefaultPosition, wxDefaultSize, wxNO_BORDER) {
        SetBackgroundStyle(wxBG_STYLE_TRANSPARENT);
    }

private:
    void OnPaint(wxPaintEvent& WXUNUSED(evt)) {
        wxPaintDC dc(this);
        wxRect rect = GetClientRect();
        rect.Deflate(1, 1);
        if (rect.width <= 0 || rect.height <= 0) return;

        std::unique_ptr<wxGraphicsContext> gc(wxGraphicsContext::Create(dc));
        if (!gc) return;

        gc->SetBrush(wxBrush(wxColour(255, 255, 255, 20)));
        gc->SetPen(wxPen(wxColour(255, 255, 255, 45), 1));
        gc->DrawRectangle(rect.x, rect.y, rect.width, rect.height);
    }

    wxDECLARE_EVENT_TABLE();
};

wxBEGIN_EVENT_TABLE(ControlCardPanel, wxPanel)
    EVT_PAINT(ControlCardPanel::OnPaint)
wxEND_EVENT_TABLE()

// ponytail: interactive synced lyrics list matching reference /play lyrics
class PlayViewCtrl::LyricsCtrl : public wxPanel {
public:
    LyricsCtrl(wxWindow* parent, wxWindowID id = wxID_ANY)
        : wxPanel(parent, id, wxDefaultPosition, wxDefaultSize, wxNO_BORDER) {
        SetBackgroundStyle(wxBG_STYLE_TRANSPARENT);
        SetCursor(wxCursor(wxCURSOR_HAND));

        Bind(wxEVT_PAINT, &LyricsCtrl::OnPaint, this);
        Bind(wxEVT_SIZE, &LyricsCtrl::OnSize, this);
        Bind(wxEVT_LEFT_UP, &LyricsCtrl::OnLeftUp, this);
        Bind(wxEVT_MOUSEWHEEL, &LyricsCtrl::OnMouseWheel, this);
    }

    void SetLyrics(const std::vector<LyricLine>& lyrics) {
        m_lyrics = lyrics;
        m_activeIndex = -1;
        m_currentScrollY = 0.0f;
        Refresh();
    }

    void SetActiveIndex(int index) {
        if (m_lyrics.empty()) return;
        if (index == m_activeIndex) return;
        m_activeIndex = index;

        if (m_userScrollHoldCount > 0) {
            Refresh();
            return;
        }

        float lineHeight = 54.0f;
        m_currentScrollY = std::max(0.0f, m_activeIndex * lineHeight);
        Refresh();
    }

    void SetOnSeek(std::function<void(uint64_t)> cb) {
        m_onSeek = std::move(cb);
    }

    void TickUserHold() {
        if (m_userScrollHoldCount > 0) {
            --m_userScrollHoldCount;
        }
    }

private:
    void OnPaint(wxPaintEvent& WXUNUSED(evt)) {
        wxPaintDC dc(this);
        wxRect client = GetClientRect();
        if (client.width <= 0 || client.height <= 0) return;

        if (m_lyrics.empty()) {
            dc.SetFont(wxFontInfo(wxSize(0, 18)).Weight(wxFONTWEIGHT_MEDIUM));
            dc.SetTextForeground(wxColour(255, 255, 255, 120));
            wxString msg = "No lyrics found";
            wxSize sz = dc.GetTextExtent(msg);
            dc.DrawText(msg, (client.width - sz.GetWidth()) / 2, (client.height - sz.GetHeight()) / 2);
            return;
        }

        std::unique_ptr<wxGraphicsContext> gc(wxGraphicsContext::Create(dc));
        if (!gc) return;

        float lineHeight = 54.0f;
        int startX = 24;
        float centerY = (client.height - lineHeight) / 2.0f;

        for (size_t i = 0; i < m_lyrics.size(); ++i) {
            float y = static_cast<float>(i) * lineHeight - m_currentScrollY + centerY;
            if (y + lineHeight < 0 || y > client.height) continue;

            bool isActive = (static_cast<int>(i) == m_activeIndex);
            std::string text = m_lyrics[i].text;
            if (text.empty()) text = "♪";

            wxString wxTxt = wxString::FromUTF8(text);
            if (isActive) {
                wxFont activeFont = wxFontInfo(wxSize(0, 22)).Bold().Family(wxFONTFAMILY_DEFAULT);
                gc->SetFont(activeFont, wxColour(255, 255, 255, 255));
            } else {
                wxFont regularFont = wxFontInfo(wxSize(0, 16)).Family(wxFONTFAMILY_DEFAULT);
                gc->SetFont(regularFont, wxColour(255, 255, 255, 115));
            }

            gc->DrawText(wxTxt, startX, y);
        }
    }

    void OnSize(wxSizeEvent& evt) {
        Refresh();
        evt.Skip();
    }

    void OnLeftUp(wxMouseEvent& evt) {
        if (m_lyrics.empty()) return;
        float lineHeight = 54.0f;
        int clientH = GetClientSize().GetHeight();
        float centerY = (clientH - lineHeight) / 2.0f;
        float clickedY = evt.GetY() + m_currentScrollY - centerY;
        int index = static_cast<int>(std::floor(clickedY / lineHeight));

        if (index >= 0 && index < static_cast<int>(m_lyrics.size())) {
            if (m_onSeek) {
                m_onSeek(m_lyrics[index].timestamp_ms);
            }
        }
    }

    void OnMouseWheel(wxMouseEvent& evt) {
        m_currentScrollY -= evt.GetWheelRotation() * 0.8f;
        m_currentScrollY = std::max(0.0f, m_currentScrollY);
        m_userScrollHoldCount = 15;
        Refresh();
    }

    std::vector<LyricLine> m_lyrics;
    int m_activeIndex = -1;
    float m_currentScrollY = 0.0f;
    int m_userScrollHoldCount = 0;
    std::function<void(uint64_t)> m_onSeek;
};

wxBEGIN_EVENT_TABLE(PlayViewCtrl, wxPanel)
    EVT_PAINT(PlayViewCtrl::OnPaint)
    EVT_SIZE(PlayViewCtrl::OnSize)
    EVT_CHAR_HOOK(PlayViewCtrl::OnCharHook)
    EVT_BUTTON(ID_PLAYVIEW_PLAY_PAUSE, PlayViewCtrl::OnPlayPause)
    EVT_BUTTON(ID_PLAYVIEW_PREV, PlayViewCtrl::OnPrev)
    EVT_BUTTON(ID_PLAYVIEW_NEXT, PlayViewCtrl::OnNext)
    EVT_BUTTON(ID_PLAYVIEW_SHUFFLE, PlayViewCtrl::OnShuffle)
    EVT_BUTTON(ID_PLAYVIEW_REPEAT, PlayViewCtrl::OnRepeat)
    EVT_BUTTON(ID_PLAYVIEW_BACK, PlayViewCtrl::OnBackBtn)
    EVT_TIMER(ID_PLAYVIEW_SYNC_TIMER, PlayViewCtrl::OnTimer)
wxEND_EVENT_TABLE()

PlayViewCtrl::PlayViewCtrl(wxWindow* parent, PlayerService* playerService, ImageService* imageService, wxWindowID id)
    : wxPanel(parent, id, wxDefaultPosition, wxDefaultSize, wxNO_BORDER),
      m_playerService(playerService),
      m_imageService(imageService),
      m_syncTimer(this, ID_PLAYVIEW_SYNC_TIMER) {
    SetBackgroundStyle(wxBG_STYLE_TRANSPARENT);
    InitUI();
}

PlayViewCtrl::~PlayViewCtrl() {
    if (m_syncTimer.IsRunning()) m_syncTimer.Stop();
}

void PlayViewCtrl::InitUI() {
    wxBoxSizer* rootSizer = new wxBoxSizer(wxVERTICAL);
    wxColour activeCol(255, 255, 255, 255);
    wxColour inactiveCol(255, 255, 255, 128);

    // 1. Top bar: Back Button
    wxBoxSizer* topBarSizer = new wxBoxSizer(wxHORIZONTAL);
    m_btnBack = new wxBitmapButton(this, ID_PLAYVIEW_BACK, wxBitmapBundle(), wxDefaultPosition, wxSize(36, 36), wxBORDER_NONE);
    m_btnBack->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnBack, Icons::Get(Icons::Back, activeCol, wxSize(24, 24)));
    m_btnBack->SetToolTip("Back (Esc)");
    topBarSizer->Add(m_btnBack, 0, wxALIGN_CENTER_VERTICAL | wxLEFT | wxTOP, 16);
    rootSizer->Add(topBarSizer, 0, wxEXPAND);

    // 2. Main split view: Left (Cover + Card), Right (Lyrics)
    m_mainSizer = new wxBoxSizer(wxHORIZONTAL);

    // Left Column
    m_leftColSizer = new wxBoxSizer(wxVERTICAL);
    m_leftColSizer->AddStretchSpacer(1);

    m_coverPanel = new CoverPanel(this, wxID_ANY, wxSize(300, 300));
    m_leftColSizer->Add(m_coverPanel, 0, wxALIGN_RIGHT | wxBOTTOM, 16);

    // Controls Glass Card
    m_cardPanel = new ControlCardPanel(this, wxID_ANY);
    m_cardPanel->SetMinSize(wxSize(300, -1));
    m_cardPanel->SetMaxSize(wxSize(300, -1));
    wxBoxSizer* cardInnerSizer = new wxBoxSizer(wxVERTICAL);

    // Card Row 1: Time, Title/Artist, Duration
    wxBoxSizer* metaRowSizer = new wxBoxSizer(wxHORIZONTAL);
    m_lblCurrentTime = new wxStaticText(m_cardPanel, wxID_ANY, "0:00", wxDefaultPosition, wxSize(48, -1), wxALIGN_LEFT);
    m_lblCurrentTime->SetForegroundColour(inactiveCol);
    m_lblCurrentTime->SetFont(wxFontInfo(wxSize(0, 11)).Family(wxFONTFAMILY_DEFAULT));

    wxBoxSizer* titleArtistSizer = new wxBoxSizer(wxVERTICAL);
    m_lblTitle = new wxStaticText(m_cardPanel, wxID_ANY, "No track playing", wxDefaultPosition, wxDefaultSize, wxALIGN_CENTER_HORIZONTAL | wxST_ELLIPSIZE_END);
    m_lblTitle->SetForegroundColour(activeCol);
    m_lblTitle->SetFont(wxFontInfo(wxSize(0, 14)).Weight(wxFONTWEIGHT_BOLD).Family(wxFONTFAMILY_DEFAULT));

    m_lblArtist = new wxStaticText(m_cardPanel, wxID_ANY, "Fluyer", wxDefaultPosition, wxDefaultSize, wxALIGN_CENTER_HORIZONTAL | wxST_ELLIPSIZE_END);
    m_lblArtist->SetForegroundColour(inactiveCol);
    m_lblArtist->SetFont(wxFontInfo(wxSize(0, 12)).Family(wxFONTFAMILY_DEFAULT));

    titleArtistSizer->Add(m_lblTitle, 0, wxALIGN_CENTER_HORIZONTAL);
    titleArtistSizer->AddSpacer(2);
    titleArtistSizer->Add(m_lblArtist, 0, wxALIGN_CENTER_HORIZONTAL);

    m_lblTotalTime = new wxStaticText(m_cardPanel, wxID_ANY, "0:00", wxDefaultPosition, wxSize(48, -1), wxALIGN_RIGHT);
    m_lblTotalTime->SetForegroundColour(inactiveCol);
    m_lblTotalTime->SetFont(wxFontInfo(wxSize(0, 11)).Family(wxFONTFAMILY_DEFAULT));

    metaRowSizer->Add(m_lblCurrentTime, 0, wxALIGN_CENTER_VERTICAL);
    metaRowSizer->Add(titleArtistSizer, 1, wxALIGN_CENTER_VERTICAL | wxLEFT | wxRIGHT, 8);
    metaRowSizer->Add(m_lblTotalTime, 0, wxALIGN_CENTER_VERTICAL);
    cardInnerSizer->Add(metaRowSizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxTOP, 16);

    // Card Row 2: Progress bar
    m_progressBar = new ProgressBarCtrl(m_cardPanel, wxID_ANY, 5, true, wxSize(280, 10));
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
    cardInnerSizer->Add(m_progressBar, 0, wxEXPAND | wxLEFT | wxRIGHT | wxTOP, 14);

    // Card Row 3: Buttons (Shuffle, Prev, Play/Pause, Next, Repeat)
    wxBoxSizer* btnRowSizer = new wxBoxSizer(wxHORIZONTAL);
    m_btnShuffle = new wxBitmapButton(m_cardPanel, ID_PLAYVIEW_SHUFFLE, wxBitmapBundle(), wxDefaultPosition, kPlayViewSmallBtnSize, wxBORDER_NONE);
    m_btnShuffle->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnShuffle, Icons::Get(Icons::Shuffle, inactiveCol, kPlayViewSmallBtnSize));

    m_btnPrev = new wxBitmapButton(m_cardPanel, ID_PLAYVIEW_PREV, wxBitmapBundle(), wxDefaultPosition, kPlayViewBtnSize, wxBORDER_NONE);
    m_btnPrev->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnPrev, Icons::Get(Icons::Prev, activeCol, kPlayViewBtnSize));

    m_btnPlayPause = new wxBitmapButton(m_cardPanel, ID_PLAYVIEW_PLAY_PAUSE, wxBitmapBundle(), wxDefaultPosition, wxSize(42, 42), wxBORDER_NONE);
    m_btnPlayPause->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnPlayPause, Icons::Get(Icons::Play, activeCol, wxSize(36, 36)));

    m_btnNext = new wxBitmapButton(m_cardPanel, ID_PLAYVIEW_NEXT, wxBitmapBundle(), wxDefaultPosition, kPlayViewBtnSize, wxBORDER_NONE);
    m_btnNext->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnNext, Icons::Get(Icons::Next, activeCol, kPlayViewBtnSize));

    m_btnRepeat = new wxBitmapButton(m_cardPanel, ID_PLAYVIEW_REPEAT, wxBitmapBundle(), wxDefaultPosition, kPlayViewSmallBtnSize, wxBORDER_NONE);
    m_btnRepeat->SetBitmapMargins(0, 0);
    SetButtonBitmap(m_btnRepeat, Icons::Get(Icons::Repeat, inactiveCol, kPlayViewSmallBtnSize));

    btnRowSizer->Add(m_btnShuffle, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 14);
    btnRowSizer->Add(m_btnPrev, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 10);
    btnRowSizer->Add(m_btnPlayPause, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 10);
    btnRowSizer->Add(m_btnNext, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 14);
    btnRowSizer->Add(m_btnRepeat, 0, wxALIGN_CENTER_VERTICAL, 0);

    cardInnerSizer->Add(btnRowSizer, 0, wxALIGN_CENTER_HORIZONTAL | wxTOP | wxBOTTOM, 16);

    m_cardPanel->SetSizer(cardInnerSizer);
    m_leftColSizer->Add(m_cardPanel, 0, wxALIGN_RIGHT);
    m_leftColSizer->AddStretchSpacer(1);

    // Add Left Column to main sizer (40% width when lyrics shown)
    m_mainSizer->AddStretchSpacer(1);
    m_mainSizer->Add(m_leftColSizer, 4, wxEXPAND | wxTOP | wxBOTTOM | wxLEFT, 16);

    // Right Column: Lyrics (60% width when lyrics shown)
    m_lyricsCtrl = new LyricsCtrl(this, wxID_ANY);
    m_lyricsCtrl->SetOnSeek([this](uint64_t timestamp_ms) {
        if (m_playerService) {
            m_playerService->Seek(timestamp_ms);
        }
    });

    m_mainSizer->Add(m_lyricsCtrl, 6, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 20);
    m_mainSizer->AddStretchSpacer(1);

    rootSizer->Add(m_mainSizer, 1, wxEXPAND);
    SetSizer(rootSizer);

    if (m_playerService) {
        m_playerService->SetOnLyricsUpdated([this]() {
            wxTheApp->CallAfter([this]() {
                if (m_playerService) {
                    m_lyricsCtrl->SetLyrics(m_playerService->GetLyrics());
                    UpdateLayoutForLyrics();
                }
            });
        });
    }

    UpdateTrack();
    UpdateState();
}

void PlayViewCtrl::UpdateLayoutForLyrics() {
    bool hasLyrics = m_playerService && m_playerService->GetLyrics().size() > 1;
    m_lyricsCtrl->Show(hasLyrics);

    // When no lyrics, left column is centered (spacers visible). When has lyrics, lyrics fill right side (60%).
    m_mainSizer->GetItem(static_cast<size_t>(0))->Show(!hasLyrics);
    m_mainSizer->GetItem(static_cast<size_t>(3))->Show(!hasLyrics);

    int align = hasLyrics ? wxALIGN_RIGHT : wxALIGN_CENTER_HORIZONTAL;
    m_leftColSizer->GetItem(m_coverPanel)->SetFlag(align | wxBOTTOM);
    m_leftColSizer->GetItem(m_cardPanel)->SetFlag(align);

    if (hasLyrics) {
        m_mainSizer->GetItem(m_leftColSizer)->SetProportion(4);
        m_mainSizer->GetItem(m_lyricsCtrl)->SetProportion(6);
    } else {
        m_mainSizer->GetItem(m_leftColSizer)->SetProportion(0);
        m_mainSizer->GetItem(m_lyricsCtrl)->SetProportion(0);
    }

    Layout();
}

void PlayViewCtrl::UpdateTrack() {
    if (!m_playerService) return;
    const auto& track = m_playerService->GetCurrentTrack();

    m_lblTitle->SetLabel(wxString::FromUTF8(track.title.empty() ? "No track playing" : track.title));
    m_lblArtist->SetLabel(wxString::FromUTF8(track.artist.empty() ? "Fluyer" : track.artist));

    const auto& state = m_playerService->GetState();
    m_lblTotalTime->SetLabel(PlayerService::FormatTime(state.duration_ms));

    OnCoverLoaded();
    m_lyricsCtrl->SetLyrics(m_playerService->GetLyrics());
    UpdateLayoutForLyrics();
    m_cardPanel->Layout();
}

void PlayViewCtrl::UpdateState() {
    if (!m_playerService) return;
    const auto& state = m_playerService->GetState();

    wxColour activeCol(255, 255, 255, 255);
    wxColour inactiveCol(255, 255, 255, 128);

    SetButtonBitmap(m_btnPlayPause, state.is_playing ? Icons::Get(Icons::Pause, activeCol, wxSize(36, 36))
                                                     : Icons::Get(Icons::Play, activeCol, wxSize(36, 36)));
    m_btnPlayPause->Refresh();

    if (state.is_playing) {
        if (!m_syncTimer.IsRunning()) {
            m_syncTimer.Start(200);
        }
    } else {
        if (m_syncTimer.IsRunning()) {
            m_syncTimer.Stop();
        }
    }

    UpdateProgress();

    SetButtonBitmap(m_btnShuffle, state.is_shuffled ? Icons::Get(Icons::Shuffle, activeCol, kPlayViewSmallBtnSize)
                                                    : Icons::Get(Icons::Shuffle, inactiveCol, kPlayViewSmallBtnSize));
    m_btnShuffle->Refresh();

    if (state.repeat_mode == FluyerRepeatMode::All) {
        SetButtonBitmap(m_btnRepeat, Icons::Get(Icons::Repeat, activeCol, kPlayViewSmallBtnSize));
    } else if (state.repeat_mode == FluyerRepeatMode::One) {
        SetButtonBitmap(m_btnRepeat, Icons::Get(Icons::RepeatOnce, activeCol, kPlayViewSmallBtnSize));
    } else {
        SetButtonBitmap(m_btnRepeat, Icons::Get(Icons::Repeat, inactiveCol, kPlayViewSmallBtnSize));
    }
    m_btnRepeat->Refresh();
}

void PlayViewCtrl::UpdateProgress() {
    if (!m_playerService) return;
    const auto& state = m_playerService->GetState();

    uint64_t pos = m_playerService->GetPosition();
    m_lblCurrentTime->SetLabel(PlayerService::FormatTime(pos));
    m_lblTotalTime->SetLabel(PlayerService::FormatTime(state.duration_ms));

    if (state.duration_ms > 0) {
        float pct = static_cast<float>(pos) / static_cast<float>(state.duration_ms);
        m_progressBar->SetPercentage(pct);
    } else {
        m_progressBar->SetPercentage(0.0f);
    }

    int activeIdx = m_playerService->GetCurrentLyricIndex(pos);
    m_lyricsCtrl->SetActiveIndex(activeIdx);
    m_lyricsCtrl->TickUserHold();
}

void PlayViewCtrl::OnCoverLoaded() {
    if (m_imageService) {
        wxBitmap cover = m_imageService->GetCurrentImage(300, GetContentScaleFactor());
        static_cast<CoverPanel*>(m_coverPanel)->SetCover(cover);
    }
}

void PlayViewCtrl::OnTimer(wxTimerEvent& WXUNUSED(evt)) {
    UpdateProgress();
}

void PlayViewCtrl::OnPlayPause(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->TogglePlay();
}

void PlayViewCtrl::OnNext(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->Next();
}

void PlayViewCtrl::OnPrev(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->Previous();
}

void PlayViewCtrl::OnShuffle(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->Shuffle();
}

void PlayViewCtrl::OnRepeat(wxCommandEvent& WXUNUSED(evt)) {
    if (m_playerService) m_playerService->CycleRepeat();
}

void PlayViewCtrl::OnBackBtn(wxCommandEvent& WXUNUSED(evt)) {
    if (m_onBack) m_onBack();
}

void PlayViewCtrl::OnCharHook(wxKeyEvent& evt) {
    if (evt.GetKeyCode() == WXK_ESCAPE) {
        if (m_onBack) {
            m_onBack();
            return;
        }
    }
    evt.Skip();
}

void PlayViewCtrl::OnPaint(wxPaintEvent& WXUNUSED(evt)) {
    wxPaintDC dc(this);
}

void PlayViewCtrl::OnSize(wxSizeEvent& evt) {
    Refresh();
    evt.Skip();
}
