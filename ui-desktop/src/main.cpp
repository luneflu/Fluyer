#include <wx/wx.h>
#include <wx/image.h>
#include <wx/stdpaths.h>
#include "fluyer_core.h"
#include "services/LibraryService.h"
#include "services/PlayerService.h"
#include "services/ImageService.h"
#include "ui/MainFrame.h"

class FluyerApp : public wxApp {
public:
    virtual bool OnInit() override;
    virtual int OnExit() override;

private:
    FluyerEngine* m_engine = nullptr;
    LibraryService m_libraryService;
    PlayerService m_playerService;
    ImageService m_imageService;
    MainFrame* m_mainFrame = nullptr;

    static void OnScanProgress(void* user_data, uintptr_t current, uintptr_t total);
    static void OnToast(void* user_data, const char* msg);
    static void OnTrackCoverLoaded(void* user_data, uintptr_t index);
    static void OnAlbumCoverLoaded(void* user_data, uintptr_t index);
    static void OnStateChanged(void* user_data, FluyerPlayerState state);
    static void OnTrackChanged(void* user_data, const char* jsonMeta, uintptr_t index);
};

wxIMPLEMENT_APP(FluyerApp);

void FluyerApp::OnScanProgress(void* user_data, uintptr_t current, uintptr_t total) {
}

void FluyerApp::OnToast(void* user_data, const char* msg) {
    auto* app = static_cast<FluyerApp*>(user_data);
    if (app && app->m_mainFrame) {
        wxTheApp->CallAfter([app]() {
            app->m_libraryService.ClearCache();
            app->m_imageService.ClearCache();
            app->m_mainFrame->RefreshViews();
        });
    }
}

void FluyerApp::OnStateChanged(void* user_data, FluyerPlayerState state) {
    auto* app = static_cast<FluyerApp*>(user_data);
    if (app && app->m_mainFrame) {
        wxTheApp->CallAfter([app, state]() {
            app->m_mainFrame->OnPlayerStateChanged(state);
        });
    }
}

void FluyerApp::OnTrackChanged(void* user_data, const char* jsonMeta, uintptr_t index) {
    auto* app = static_cast<FluyerApp*>(user_data);
    if (app && app->m_mainFrame) {
        std::string jsonStr = jsonMeta ? jsonMeta : "";
        wxTheApp->CallAfter([app, jsonStr, index]() {
            app->m_mainFrame->OnTrackChanged(jsonStr, index);
        });
    }
}

void FluyerApp::OnTrackCoverLoaded(void* user_data, uintptr_t index) {
    auto* app = static_cast<FluyerApp*>(user_data);
    if (app && app->m_mainFrame) {
        wxTheApp->CallAfter([app, index]() {
            app->m_mainFrame->OnTrackCoverLoaded(index);
        });
    }
}

void FluyerApp::OnAlbumCoverLoaded(void* user_data, uintptr_t index) {
    auto* app = static_cast<FluyerApp*>(user_data);
    if (app && app->m_mainFrame) {
        wxTheApp->CallAfter([app, index]() {
            app->m_mainFrame->OnAlbumCoverLoaded(index);
        });
    }
}

bool FluyerApp::OnInit() {
    if (!wxApp::OnInit()) return false;

    wxInitAllImageHandlers();

    wxStandardPaths& stdPaths = wxStandardPaths::Get();
    wxString appSupportDir = "/Users/alvindimas05/Library/Application Support/org.alvindimas05.fluyer";
    wxString cacheDir = stdPaths.GetUserLocalDataDir() + "/cache";

    FluyerCallbacks callbacks = { 0 };
    callbacks.user_data = this;
    callbacks.on_state_changed = FluyerApp::OnStateChanged;
    callbacks.on_track_changed = FluyerApp::OnTrackChanged;
    callbacks.on_scan_progress = FluyerApp::OnScanProgress;
    callbacks.on_toast = FluyerApp::OnToast;
    callbacks.on_track_cover_loaded = FluyerApp::OnTrackCoverLoaded;
    callbacks.on_album_cover_loaded = FluyerApp::OnAlbumCoverLoaded;

    m_engine = fluyer_init(appSupportDir.c_str(), cacheDir.c_str(), callbacks);

    m_libraryService.SetEngine(m_engine);
    m_playerService.SetEngine(m_engine);
    m_imageService.SetEngine(m_engine);

    m_mainFrame = new MainFrame(&m_libraryService, &m_playerService, &m_imageService);
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
