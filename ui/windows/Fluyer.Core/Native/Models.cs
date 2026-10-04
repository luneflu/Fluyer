using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fluyer.Core.Native;

/// <summary>
/// Core transport repeat mode. Serde serializes the Rust unit-variant enum as
/// <c>"None"</c>/<c>"All"</c>/<c>"One"</c> strings — hence the custom converter.
/// </summary>
[JsonConverter(typeof(RepeatModeConverter))]
public enum RepeatMode
{
    None = 0,
    All = 1,
    One = 2,
}

internal sealed class RepeatModeConverter : JsonConverter<RepeatMode>
{
    public override RepeatMode Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        // Serde may emit either the variant name ("All") or, if ever configured
        // otherwise, the discriminant number.
        if (reader.TokenType == JsonTokenType.String)
        {
            var name = reader.GetString();
            return name switch
            {
                "All" => RepeatMode.All,
                "One" => RepeatMode.One,
                _ => RepeatMode.None,
            };
        }
        if (reader.TokenType == JsonTokenType.Number && reader.TryGetByte(out var discriminant))
        {
            return discriminant switch
            {
                1 => RepeatMode.All,
                2 => RepeatMode.One,
                _ => RepeatMode.None,
            };
        }
        throw new JsonException($"Unexpected token {reader.TokenType} for RepeatMode.");
    }

    public override void Write(Utf8JsonWriter writer, RepeatMode value, JsonSerializerOptions options)
    {
        writer.WriteStringValue(value switch
        {
            RepeatMode.All => "All",
            RepeatMode.One => "One",
            _ => "None",
        });
    }
}

/// <summary>Mirrors <c>PlayerBarViewModel</c> in <c>view_models/player_bar.rs</c>.</summary>
public sealed record PlayerBarViewModel(
    [property: JsonPropertyName("track_index")] long TrackIndex,
    [property: JsonPropertyName("title")] string Title,
    [property: JsonPropertyName("artist")] string Artist,
    [property: JsonPropertyName("album")] string Album,
    [property: JsonPropertyName("position_ms")] ulong PositionMs,
    [property: JsonPropertyName("duration_ms")] ulong DurationMs,
    [property: JsonPropertyName("progress_pct")] float ProgressPct,
    [property: JsonPropertyName("time_label")] string TimeLabel,
    [property: JsonPropertyName("is_playing")] bool IsPlaying,
    [property: JsonPropertyName("repeat_mode")] RepeatMode RepeatMode,
    [property: JsonPropertyName("is_shuffled")] bool IsShuffled,
    [property: JsonPropertyName("volume")] float Volume);

/// <summary>Mirrors <c>TrackItemViewModel</c> in <c>view_models/track_item.rs</c>.</summary>
public sealed record TrackItemViewModel(
    [property: JsonPropertyName("index")] ulong Index,
    [property: JsonPropertyName("path")] string Path,
    [property: JsonPropertyName("title")] string Title,
    [property: JsonPropertyName("artist")] string Artist,
    [property: JsonPropertyName("album")] string Album,
    [property: JsonPropertyName("duration_ms")] ulong DurationMs,
    [property: JsonPropertyName("duration_formatted")] string DurationFormatted,
    [property: JsonPropertyName("is_current")] bool IsCurrent);

/// <summary>Mirrors <c>AlbumCardViewModel</c> in <c>view_models/album_card.rs</c>.</summary>
public sealed record AlbumCardViewModel(
    [property: JsonPropertyName("index")] ulong Index,
    [property: JsonPropertyName("name")] string Name,
    [property: JsonPropertyName("artist")] string Artist,
    [property: JsonPropertyName("year")] string Year,
    [property: JsonPropertyName("track_count")] ulong TrackCount,
    [property: JsonPropertyName("track_count_label")] string TrackCountLabel);

/// <summary>Mirrors <c>AlbumDetailViewModel</c> in <c>view_models/album_detail.rs</c>.</summary>
public sealed record AlbumDetailViewModel(
    [property: JsonPropertyName("header")] AlbumCardViewModel Header,
    [property: JsonPropertyName("duration_ms")] ulong DurationMs,
    [property: JsonPropertyName("total_duration_formatted")] string TotalDurationFormatted,
    [property: JsonPropertyName("subtitle")] string Subtitle,
    [property: JsonPropertyName("tracks")] List<TrackItemViewModel> Tracks);

/// <summary>Mirrors <c>LyricLine</c> in <c>view_models/common.rs</c>.</summary>
public sealed record LyricLine(
    [property: JsonPropertyName("timestamp_ms")] ulong TimestampMs,
    [property: JsonPropertyName("text")] string Text);

/// <summary>Mirrors <c>ColorRgb</c> in <c>view_models/common.rs</c>.</summary>
public sealed record ColorRgb(
    [property: JsonPropertyName("r")] byte R,
    [property: JsonPropertyName("g")] byte G,
    [property: JsonPropertyName("b")] byte B);

/// <summary>Mirrors <c>PlayViewModel</c> in <c>view_models/play_view.rs</c>.</summary>
public sealed record PlayViewModel(
    [property: JsonPropertyName("track")] TrackItemViewModel? Track,
    [property: JsonPropertyName("lyrics")] List<LyricLine> Lyrics,
    [property: JsonPropertyName("current_lyric_index")] int CurrentLyricIndex,
    [property: JsonPropertyName("palette")] List<ColorRgb> Palette);

/// <summary>Mirrors <c>ScanStatusViewModel</c> in <c>view_models/scan_status.rs</c>.</summary>
public sealed record ScanStatusViewModel(
    [property: JsonPropertyName("is_scanning")] bool IsScanning,
    [property: JsonPropertyName("current")] ulong Current,
    [property: JsonPropertyName("total")] ulong Total,
    [property: JsonPropertyName("progress_pct")] float ProgressPct,
    [property: JsonPropertyName("status_label")] string StatusLabel);

/// <summary>Mirrors <c>AnimatedBackgroundFrame</c> in <c>view_models/common.rs</c>.</summary>
public sealed record AnimatedBackgroundFrame(
    [property: JsonPropertyName("rgba")] byte[] Rgba,
    [property: JsonPropertyName("width")] uint Width,
    [property: JsonPropertyName("height")] uint Height);
