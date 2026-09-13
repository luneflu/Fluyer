#pragma once
#include <wx/wx.h>
#include <wx/slider.h>
#include <wx/mstream.h>
#include "fluyer_core.h"

class PlayerBarCtrl : public wxPanel {
public:
    PlayerBarCtrl(wxWindow* parent, wxWindowID id = wxID_ANY);

    void SetEngine(FluyerEngine* engine);
    void UpdateState(const FluyerPlayerState& state);
    void UpdateTrack(const wxString& title, const wxString& artist, const wxString& album, uintptr_t trackIdx);
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

    FluyerEngine* m_engine = nullptr;
    FluyerPlayerState m_state = { -1, 0, 0, false, FluyerRepeatMode::None, false };

    wxString m_title = "No track playing";
    wxString m_artist = "Fluyer";
    wxString m_album = "";
    wxBitmap m_coverThumb;
    uintptr_t m_currentTrackIdx = static_cast<uintptr_t>(-1);

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
    wxStaticText* m_volIcon = nullptr;
    wxSlider* m_volSlider = nullptr;

    wxDECLARE_EVENT_TABLE();
};
