#include <wx/wx.h>
#include <wx/image.h>
#include <wx/stdpaths.h>
#include <wx/file.h>
#include "fluyer_core.h"
#include "MainFrame.h"

class FluyerApp : public wxApp {
public:
    virtual bool OnInit() override;
    virtual int OnExit() override;

private:
    FluyerEngine* m_engine = nullptr;
    MainFrame* m_mainFrame = nullptr;

    static void OnScanProgress(void* user_data, uintptr_t current, uintptr_t total);
    static void OnToast(void* user_data, const char* msg);
};

wxIMPLEMENT_APP(FluyerApp);

void FluyerApp::OnScanProgress(void* user_data, uintptr_t current, uintptr_t total) {
}

void FluyerApp::OnToast(void* user_data, const char* msg) {
    auto* app = static_cast<FluyerApp*>(user_data);
    if (app && app->m_mainFrame) {
        wxTheApp->CallAfter([app]() {
            app->m_mainFrame->RefreshViews();
        });
    }
}

bool FluyerApp::OnInit() {
    if (!wxApp::OnInit()) return false;

    wxInitAllImageHandlers();

    // Use exact Tauri app data directory
    wxStandardPaths& stdPaths = wxStandardPaths::Get();
    wxString appSupportDir = "/Users/alvindimas05/Library/Application Support/org.alvindimas05.fluyer";
    wxString cacheDir = stdPaths.GetUserLocalDataDir() + "/cache";

    FluyerCallbacks callbacks = { 0 };
    callbacks.user_data = this;
    callbacks.on_scan_progress = FluyerApp::OnScanProgress;
    callbacks.on_toast = FluyerApp::OnToast;

    m_engine = fluyer_init(appSupportDir.c_str(), cacheDir.c_str(), callbacks);

    m_mainFrame = new MainFrame(m_engine);
    m_mainFrame->Show(true);
    m_mainFrame->RefreshViews();

    return true;
}

int FluyerApp::OnExit() {
    if (m_engine) {
        fluyer_free(m_engine);
        m_engine = nullptr;
    }
    return wxApp::OnExit();
}
