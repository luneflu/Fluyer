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
    m_lyrics.clear();
    LoadLyrics();
}

void PlayerService::ClearLyrics() {
    m_lyrics.clear();
    if (m_onLyricsUpdated) m_onLyricsUpdated();
}

void PlayerService::LoadLyrics() {
    if (!m_engine) return;
    char* lrc = fluyer_player_get_lyrics(m_engine);
    if (lrc) {
        SetLyrics(lrc);
        fluyer_string_free(lrc);
    }
}

void PlayerService::SetLyrics(const std::string& lrcText) {
    m_lyrics.clear();
    if (lrcText.empty()) {
        if (m_onLyricsUpdated) m_onLyricsUpdated();
        return;
    }

    std::istringstream stream(lrcText);
    std::string line;
    while (std::getline(stream, line)) {
        if (!line.empty() && line.back() == '\r') line.pop_back();

        size_t openBracket = line.find('[');
        size_t closeBracket = line.find(']');
        if (openBracket == std::string::npos || closeBracket == std::string::npos || closeBracket <= openBracket) {
            continue;
        }

        std::string tag = line.substr(openBracket + 1, closeBracket - openBracket - 1);
        size_t colon = tag.find(':');
        if (colon == std::string::npos) continue;

        try {
            int minutes = std::stoi(tag.substr(0, colon));
            std::string secPart = tag.substr(colon + 1);
            double seconds = std::stod(secPart);
            uint64_t timestamp_ms = static_cast<uint64_t>((minutes * 60.0 + seconds) * 1000.0);

            std::string text = line.substr(closeBracket + 1);
            size_t start = text.find_first_not_of(" \t");
            if (start != std::string::npos) {
                size_t end = text.find_last_not_of(" \t");
                text = text.substr(start, end - start + 1);
            } else {
                text.clear();
            }

            if (m_lyrics.empty() && timestamp_ms > 5000) {
                m_lyrics.push_back({0, ""});
            }
            m_lyrics.push_back({timestamp_ms, text});
        } catch (...) {
            continue;
        }
    }

    std::stable_sort(m_lyrics.begin(), m_lyrics.end(), [](const LyricLine& a, const LyricLine& b) {
        return a.timestamp_ms < b.timestamp_ms;
    });

    if (m_onLyricsUpdated) m_onLyricsUpdated();
}

int PlayerService::GetCurrentLyricIndex(uint64_t position_ms) const {
    if (m_lyrics.empty()) return -1;
    if (position_ms < m_lyrics[0].timestamp_ms) return 0;
    for (size_t i = 0; i < m_lyrics.size(); ++i) {
        if (position_ms < m_lyrics[i].timestamp_ms) {
            return static_cast<int>(i) - 1;
        }
    }
    return static_cast<int>(m_lyrics.size()) - 1;
}

std::string PlayerService::FormatTime(uint64_t ms) {
    uint64_t totalSec = ms / 1000;
    uint64_t min = totalSec / 60;
    uint64_t sec = totalSec % 60;
    std::ostringstream ss;
    ss << min << ":" << std::setw(2) << std::setfill('0') << sec;
    return ss.str();
}
