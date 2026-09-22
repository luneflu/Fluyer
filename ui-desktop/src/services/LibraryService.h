#pragma once
#include <vector>
#include <string>
#include <unordered_map>
#include "fluyer_core.h"
#include "models/Track.h"
#include "models/Album.h"

class LibraryService {
public:
    explicit LibraryService(FluyerEngine* engine = nullptr);

    void SetEngine(FluyerEngine* engine);
    uintptr_t GetTrackCount() const;
    uintptr_t GetAlbumCount() const;

    Track GetTrack(uintptr_t index);
    Album GetAlbum(uintptr_t index);

    void PlayTrack(uintptr_t index);
    void PlayAlbum(uintptr_t index);
    void PlayAlbumTrack(uintptr_t albumIndex, uintptr_t trackIndex);
    void QueueAlbum(uintptr_t index);
    void ShuffleAlbum(uintptr_t index);
    void ScanDirectories(const std::vector<std::string>& paths);
    void ClearCache();

private:
    FluyerEngine* m_engine = nullptr;
    std::unordered_map<uintptr_t, Track> m_trackCache;
    std::unordered_map<uintptr_t, Album> m_albumCache;
};
