#pragma once
#include <string>
#include <cstdint>
#include "fluyer_core.h"
#include "models/Track.h"

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
};
