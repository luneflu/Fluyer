#include <cassert>
#include <iostream>
#include "services/PlayerService.h"

int main() {
    PlayerService player;

    std::string sampleLrc =
        "[00:00.00] Intro\n"
        "[00:10.50] First line of lyrics\n"
        "[00:25.00] Second line of lyrics\n"
        "[01:05.20] Final chorus\n";

    player.SetLyrics(sampleLrc);
    const auto& lyrics = player.GetLyrics();

    assert(lyrics.size() == 4);
    assert(lyrics[0].timestamp_ms == 0);
    assert(lyrics[0].text == "Intro");
    assert(lyrics[1].timestamp_ms == 10500);
    assert(lyrics[1].text == "First line of lyrics");
    assert(lyrics[2].timestamp_ms == 25000);
    assert(lyrics[2].text == "Second line of lyrics");
    assert(lyrics[3].timestamp_ms == 65200);
    assert(lyrics[3].text == "Final chorus");

    // Test active lyric index lookup
    assert(player.GetCurrentLyricIndex(0) == 0);
    assert(player.GetCurrentLyricIndex(5000) == 0);
    assert(player.GetCurrentLyricIndex(10500) == 1);
    assert(player.GetCurrentLyricIndex(20000) == 1);
    assert(player.GetCurrentLyricIndex(25000) == 2);
    assert(player.GetCurrentLyricIndex(65199) == 2);
    assert(player.GetCurrentLyricIndex(65200) == 3);
    assert(player.GetCurrentLyricIndex(100000) == 3);

    // Test auto-inserting leading empty line when first lyric starts after 5 seconds
    PlayerService player2;
    std::string lateLrc = "[00:15.00] Late start lyric\n";
    player2.SetLyrics(lateLrc);
    const auto& lateLyrics = player2.GetLyrics();
    assert(lateLyrics.size() == 2);
    assert(lateLyrics[0].timestamp_ms == 0);
    assert(lateLyrics[0].text == "");
    assert(lateLyrics[1].timestamp_ms == 15000);
    assert(lateLyrics[1].text == "Late start lyric");

    std::cout << "All lyrics tests passed successfully!\n";
    return 0;
}
