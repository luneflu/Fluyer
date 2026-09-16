#include "PlayerService.h"
#include <iomanip>
#include <sstream>

PlayerService::PlayerService(FluyerEngine* engine)
    : m_engine(engine) {}

void PlayerService::SetEngine(FluyerEngine* engine) {
    m_engine = engine;
}

void PlayerService::TogglePlay() {
    if (m_engine) fluyer_player_toggle_play(m_engine);
}

void PlayerService::Play() {
    if (m_engine) fluyer_player_play(m_engine);
}

void PlayerService::Pause() {
    if (m_engine) fluyer_player_pause(m_engine);
}

void PlayerService::Next() {
    if (m_engine) fluyer_player_next(m_engine);
}

void PlayerService::Previous() {
    if (m_engine) fluyer_player_previous(m_engine);
}

void PlayerService::Shuffle() {
    if (m_engine) fluyer_player_shuffle(m_engine);
}

void PlayerService::CycleRepeat() {
    if (!m_engine) return;
    FluyerRepeatMode nextMode;
    if (m_state.repeat_mode == FluyerRepeatMode::None) {
        nextMode = FluyerRepeatMode::All;
    } else if (m_state.repeat_mode == FluyerRepeatMode::All) {
        nextMode = FluyerRepeatMode::One;
    } else {
        nextMode = FluyerRepeatMode::None;
    }
    fluyer_player_set_repeat(m_engine, nextMode);
}

void PlayerService::Seek(uint64_t position_ms) {
    if (m_engine) fluyer_player_seek(m_engine, position_ms);
}

void PlayerService::SeekPercent(float percent0to1) {
    if (!m_engine || m_state.duration_ms == 0) return;
    uint64_t pos = static_cast<uint64_t>(percent0to1 * m_state.duration_ms);
    Seek(pos);
}

void PlayerService::SetVolume(float volume) {
    if (m_engine) fluyer_player_set_volume(m_engine, volume);
}

void PlayerService::RequestSync() {
    if (m_engine) fluyer_player_request_sync(m_engine);
}

void PlayerService::UpdateState(const FluyerPlayerState& state) {
    m_state = state;
}

uint64_t PlayerService::GetPosition() const {
    if (m_engine) {
        return fluyer_player_get_position(m_engine);
    }
    return m_state.position_ms;
}

void PlayerService::UpdateTrack(const std::string& jsonMeta, uintptr_t index) {
    m_currentTrack = Track::FromJSON(jsonMeta);
    m_currentTrackIndex = index;
}

std::string PlayerService::FormatTime(uint64_t ms) {
    uint64_t totalSec = ms / 1000;
    uint64_t min = totalSec / 60;
    uint64_t sec = totalSec % 60;
    std::ostringstream ss;
    ss << min << ":" << std::setw(2) << std::setfill('0') << sec;
    return ss.str();
}
