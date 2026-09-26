#pragma once
#include <wx/wx.h>
#include <wx/settings.h>
#include <algorithm>
#include <cmath>
#include <functional>
#include <string>

// ponytail: custom lightweight progress bar matching ProgressBar.svelte
class ProgressBarCtrl : public wxPanel {
public:
    ProgressBarCtrl(wxWindow* parent, wxWindowID id = wxID_ANY, int barHeight = 5,
                    bool showTooltip = false, const wxSize& size = wxDefaultSize)
        : wxPanel(parent, id, wxDefaultPosition, size, wxNO_BORDER),
          m_barHeight(barHeight),
          m_showTooltip(showTooltip) {
        SetBackgroundStyle(wxBG_STYLE_TRANSPARENT);
        SetCursor(wxCursor(wxCURSOR_HAND));

        Bind(wxEVT_PAINT, &ProgressBarCtrl::OnPaint, this);
        Bind(wxEVT_LEFT_DOWN, &ProgressBarCtrl::OnLeftDown, this);
        Bind(wxEVT_LEFT_UP, &ProgressBarCtrl::OnLeftUp, this);
        Bind(wxEVT_MOTION, &ProgressBarCtrl::OnMotion, this);
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
        wxPaintDC dc(this);

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
};
