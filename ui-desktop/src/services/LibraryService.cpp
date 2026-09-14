#include "LibraryService.h"

LibraryService::LibraryService(FluyerEngine* engine)
    : m_engine(engine) {}

void LibraryService::SetEngine(FluyerEngine* engine) {
    m_engine = engine;
    ClearCache();
}

uintptr_t LibraryService::GetTrackCount() const {
    if (!m_engine) return 0;
    return fluyer_library_get_count(m_engine);
}

uintptr_t LibraryService::GetAlbumCount() const {
    if (!m_engine) return 0;
    return fluyer_library_get_album_count(m_engine);
}

Track LibraryService::GetTrack(uintptr_t index) {
    auto it = m_trackCache.find(index);
    if (it != m_trackCache.end()) {
        return it->second;
    }

    if (!m_engine) return Track{};

    char* json = fluyer_library_get_track_json(m_engine, index);
    Track t;
    if (json) {
        t = Track::FromJSON(json);
        fluyer_string_free(json);
    }
    m_trackCache[index] = t;
    return t;
}

Album LibraryService::GetAlbum(uintptr_t index) {
    auto it = m_albumCache.find(index);
    if (it != m_albumCache.end()) {
        return it->second;
    }

    if (!m_engine) return Album{};

    char* json = fluyer_library_get_album_json(m_engine, index);
    Album a;
    if (json) {
        a = Album::FromJSON(json);
        fluyer_string_free(json);
    }
    m_albumCache[index] = a;
    return a;
}

void LibraryService::PlayTrack(uintptr_t index) {
    if (m_engine) {
        fluyer_library_play_index(m_engine, index);
    }
}

void LibraryService::ScanDirectories(const std::vector<std::string>& paths) {
    if (!m_engine || paths.empty()) return;
    std::vector<const char*> c_paths;
    c_paths.reserve(paths.size());
    for (const auto& p : paths) {
        c_paths.push_back(p.c_str());
    }
    fluyer_library_scan(m_engine, c_paths.data(), c_paths.size());
}

void LibraryService::ClearCache() {
    m_trackCache.clear();
    m_albumCache.clear();
}
