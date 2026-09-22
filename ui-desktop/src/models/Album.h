#pragma once
#include <string>
#include <vector>
#include <cstdint>
#include <iomanip>
#include <sstream>
#include <nlohmann/json.hpp>
#include "Track.h"

struct Album {
    std::string name = "Unknown Album";
    std::string artist = "Unknown Artist";
    std::string year = "";
    uint64_t duration_ms = 0;
    std::vector<Track> tracks;

    std::string FormatDuration() const {
        uint64_t total_sec = duration_ms / 1000;
        uint64_t min = total_sec / 60;
        uint64_t sec = total_sec % 60;
        std::ostringstream ss;
        ss << min << ":" << std::setw(2) << std::setfill('0') << sec;
        return ss.str();
    }

    std::string GetLabel() const {
        std::vector<std::string> parts;
        if (!name.empty()) parts.push_back(name);
        if (!artist.empty()) parts.push_back(artist);
        if (!year.empty()) parts.push_back(year);
        if (duration_ms > 0) parts.push_back(FormatDuration());

        std::string res;
        for (size_t i = 0; i < parts.size(); ++i) {
            if (i > 0) res += " • ";
            res += parts[i];
        }
        return res;
    }

    static Album FromJSON(const std::string& jsonStr) {
        Album a;
        try {
            auto j = nlohmann::json::parse(jsonStr);
            if (j.is_array() && !j.empty()) {
                const auto& first = j[0];
                if (first.contains("album") && !first["album"].is_null()) {
                    a.name = first["album"].get<std::string>();
                }
                if (first.contains("albumArtist") && !first["albumArtist"].is_null()) {
                    a.artist = first["albumArtist"].get<std::string>();
                } else if (first.contains("artist") && !first["artist"].is_null()) {
                    a.artist = first["artist"].get<std::string>();
                }
                if (first.contains("date") && !first["date"].is_null()) {
                    std::string d = first["date"].get<std::string>();
                    if (d.length() >= 4) {
                        a.year = d.substr(0, 4);
                    }
                }
                for (const auto& item : j) {
                    Track t;
                    if (item.contains("title") && !item["title"].is_null()) t.title = item["title"].get<std::string>();
                    if (item.contains("artist") && !item["artist"].is_null()) t.artist = item["artist"].get<std::string>();
                    if (item.contains("album") && !item["album"].is_null()) t.album = item["album"].get<std::string>();
                    if (item.contains("duration") && !item["duration"].is_null()) {
                        t.duration_ms = item["duration"].get<uint64_t>();
                        a.duration_ms += t.duration_ms;
                    }
                    a.tracks.push_back(t);
                }
            }
        } catch (...) {}
        return a;
    }
};
