#pragma once
#include <string>
#include <cstdint>
#include <vector>
#include <functional>
#include "fluyer_core.h"
#include "models/Track.h"

struct LyricLine {
    uint64_t timestamp_ms = 0;
    std::string text;
};

class PlayerService {
public:
    explicit PlayerService(FluyerEngine* engine = nullptr);

    void SetEngine(FluyerEngine* engine);

    void TogglePlay();
    void Play();
    void Pause();
    void Next();
    void Previous();
    void Shuffle();
    void CycleRepeat();
    void Seek(uint64_t position_ms);
    void SeekPercent(float percent0to1);
    void SetVolume(float volume);
    void RequestSync();

    void UpdateState(const FluyerPlayerState& state);
    void UpdateTrack(const std::string& jsonMeta, uintptr_t index);

    void LoadLyrics();
    void SetLyrics(const std::string& lrcText);
    void ClearLyrics();
    const std::vector<LyricLine>& GetLyrics() const { return m_lyrics; }
    int GetCurrentLyricIndex(uint64_t position_ms) const;
    void SetOnLyricsUpdated(std::function<void()> cb) { m_onLyricsUpdated = std::move(cb); }

    uint64_t GetPosition() const;
    const FluyerPlayerState& GetState() const { return m_state; }
    const Track& GetCurrentTrack() const { return m_currentTrack; }
    uintptr_t GetCurrentTrackIndex() const { return m_currentTrackIndex; }

    static std::string FormatTime(uint64_t ms);

private:
    FluyerEngine* m_engine = nullptr;
    FluyerPlayerState m_state = { -1, 0, 0, false, FluyerRepeatMode::None, false };
    Track m_currentTrack;
    uintptr_t m_currentTrackIndex = static_cast<uintptr_t>(-1);
    std::vector<LyricLine> m_lyrics;
    std::function<void()> m_onLyricsUpdated;
};
