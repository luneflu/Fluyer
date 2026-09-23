#pragma once
#include <string>
#include <cstdint>
#include <iomanip>
#include <sstream>
#include <nlohmann/json.hpp>

struct Track {
    std::string path = "";
    std::string title = "Unknown Title";
    std::string artist = "Unknown Artist";
    std::string album = "";
    uint64_t duration_ms = 0;

    std::string FormatDuration() const {
        uint64_t total_sec = duration_ms / 1000;
        std::ostringstream ss;
        ss << (total_sec / 60) << ":" << std::setw(2) << std::setfill('0') << (total_sec % 60);
        return ss.str();
    }

    static Track FromJSON(const std::string& jsonStr) {
        Track t;
        try {
            auto j = nlohmann::json::parse(jsonStr);
            if (j.contains("path") && !j["path"].is_null()) t.path = j["path"].get<std::string>();
            if (j.contains("title") && !j["title"].is_null()) t.title = j["title"].get<std::string>();
            if (j.contains("artist") && !j["artist"].is_null()) t.artist = j["artist"].get<std::string>();
            if (j.contains("album") && !j["album"].is_null()) t.album = j["album"].get<std::string>();
            if (j.contains("duration") && !j["duration"].is_null()) t.duration_ms = j["duration"].get<uint64_t>();
        } catch (...) {}
        return t;
    }
};
