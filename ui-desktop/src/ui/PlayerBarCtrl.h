#pragma once
#include <wx/wx.h>
#include <wx/statbmp.h>
#include <wx/bmpbuttn.h>
#include <wx/timer.h>
#include <functional>
#include "services/PlayerService.h"
#include "services/ImageService.h"

class ProgressBarCtrl;
class PillPanel;

class PlayerBarCtrl : public wxPanel {
public:
    PlayerBarCtrl(wxWindow* parent, PlayerService* playerService, ImageService* imageService, wxWindowID id = wxID_ANY);
    ~PlayerBarCtrl() override;

    void UpdateState();
    void UpdateTrack();
    void OnCoverLoaded();
    void SetOnCoverClicked(std::function<void()> cb) { m_onCoverClicked = std::move(cb); }

private:
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnPlayPause(wxCommandEvent& evt);
    void OnNext(wxCommandEvent& evt);
    void OnPrev(wxCommandEvent& evt);
    void OnShuffle(wxCommandEvent& evt);
    void OnRepeat(wxCommandEvent& evt);
    void OnVolumeBtn(wxCommandEvent& evt);
    void OnTimer(wxTimerEvent& evt);
    void UpdateProgress();
    void UpdateVolumeIcon();

    PlayerService* m_playerService = nullptr;
    ImageService* m_imageService = nullptr;
    wxTimer m_progressTimer;

    ProgressBarCtrl* m_progressBar = nullptr;
    PillPanel* m_pillPanel = nullptr;

    wxButton* m_btnPrev = nullptr;
    wxButton* m_btnPlayPause = nullptr;
    wxButton* m_btnNext = nullptr;

    wxStaticBitmap* m_coverView = nullptr;
    wxStaticText* m_lblTitle = nullptr;
    wxStaticText* m_lblArtist = nullptr;

    wxButton* m_btnRepeat = nullptr;
    wxButton* m_btnShuffle = nullptr;
    wxButton* m_btnVol = nullptr;
    ProgressBarCtrl* m_volBar = nullptr;
    float m_previousVolume = 1.0f;
    std::function<void()> m_onCoverClicked;

    wxDECLARE_EVENT_TABLE();
};
