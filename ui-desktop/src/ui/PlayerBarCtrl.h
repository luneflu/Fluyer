#pragma once
#include <wx/wx.h>
#include <wx/slider.h>
#include <wx/statbmp.h>
#include <wx/bmpbuttn.h>
#include <wx/timer.h>
#include "services/PlayerService.h"
#include "services/ImageService.h"

class PlayerBarCtrl : public wxPanel {
public:
    PlayerBarCtrl(wxWindow* parent, PlayerService* playerService, ImageService* imageService, wxWindowID id = wxID_ANY);
    ~PlayerBarCtrl() override;

    void UpdateState();
    void UpdateTrack();
    void OnCoverLoaded();

private:
    void OnPaint(wxPaintEvent& evt);
    void OnSize(wxSizeEvent& evt);
    void OnPlayPause(wxCommandEvent& evt);
    void OnNext(wxCommandEvent& evt);
    void OnPrev(wxCommandEvent& evt);
    void OnShuffle(wxCommandEvent& evt);
    void OnRepeat(wxCommandEvent& evt);
    void OnSeek(wxCommandEvent& evt);
    void OnVolume(wxCommandEvent& evt);
    void OnTimer(wxTimerEvent& evt);
    void UpdateProgress();

    PlayerService* m_playerService = nullptr;
    ImageService* m_imageService = nullptr;
    wxTimer m_progressTimer;

    wxSlider* m_seekSlider = nullptr;
    wxStaticText* m_lblTimePos = nullptr;
    wxStaticText* m_lblTimeDur = nullptr;

    wxPanel* m_pillPanel = nullptr;

    wxButton* m_btnPrev = nullptr;
    wxButton* m_btnPlayPause = nullptr;
    wxButton* m_btnNext = nullptr;

    wxStaticBitmap* m_coverView = nullptr;
    wxStaticText* m_lblTitle = nullptr;
    wxStaticText* m_lblArtist = nullptr;

    wxButton* m_btnRepeat = nullptr;
    wxButton* m_btnShuffle = nullptr;
    wxStaticBitmap* m_volIcon = nullptr;
    wxSlider* m_volSlider = nullptr;

    wxDECLARE_EVENT_TABLE();
};
