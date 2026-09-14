#pragma once
#include <string>
#include <nlohmann/json.hpp>

struct Album {
    std::string name = "Unknown Album";
    std::string artist = "Unknown Artist";

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
            }
        } catch (...) {}
        return a;
    }
};
