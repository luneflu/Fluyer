#pragma once
#include <wx/wx.h>
#include <wx/timer.h>
#include <vector>
#include <functional>
#include "services/PlayerService.h"
#include "services/ImageService.h"

class ProgressBarCtrl;

class PlayViewCtrl : public wxPanel {
public:
    PlayViewCtrl(wxWindow* parent, PlayerService* playerService, ImageService* imageService, wxWindowID id = wxID_ANY);
    ~PlayViewCtrl() override;

    void UpdateTrack();
    void UpdateState();
    void UpdateProgress();
    void OnCoverLoaded();
    void SetOnBack(std::function<void()> cb) { m_onBack = std::move(cb); }

private:
    void InitUI();
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnCharHook(wxKeyEvent& evt);
    void OnTimer(wxTimerEvent& evt);

    // Player action handlers
    void OnPlayPause(wxCommandEvent& evt);
    void OnNext(wxCommandEvent& evt);
    void OnPrev(wxCommandEvent& evt);
    void OnShuffle(wxCommandEvent& evt);
    void OnRepeat(wxCommandEvent& evt);
    void OnBackBtn(wxCommandEvent& evt);

    void UpdateLayoutForLyrics();

    PlayerService* m_playerService = nullptr;
    ImageService* m_imageService = nullptr;
    std::function<void()> m_onBack;

    wxTimer m_syncTimer;

    // UI Controls - Left side
    wxBoxSizer* m_mainSizer = nullptr;
    wxBoxSizer* m_leftColSizer = nullptr;
    wxPanel* m_coverPanel = nullptr;
    wxPanel* m_cardPanel = nullptr;
    wxBitmap m_coverBitmap;

    wxStaticText* m_lblCurrentTime = nullptr;
    wxStaticText* m_lblTitle = nullptr;
    wxStaticText* m_lblArtist = nullptr;
    wxStaticText* m_lblTotalTime = nullptr;
    ProgressBarCtrl* m_progressBar = nullptr;

    wxButton* m_btnBack = nullptr;
    wxButton* m_btnShuffle = nullptr;
    wxButton* m_btnPrev = nullptr;
    wxButton* m_btnPlayPause = nullptr;
    wxButton* m_btnNext = nullptr;
    wxButton* m_btnRepeat = nullptr;

    // UI Controls - Right side (Lyrics)
    class LyricsCtrl;
    LyricsCtrl* m_lyricsCtrl = nullptr;

    wxDECLARE_EVENT_TABLE();
};
