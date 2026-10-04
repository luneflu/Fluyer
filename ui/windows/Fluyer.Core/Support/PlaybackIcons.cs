using Fluyer.Core.Native;

namespace Fluyer.Core.Support;

/// <summary>
/// Port of <c>PlaybackIcons</c> (<c>ui/macos/Sources/Support/PlaybackIcons.swift</c>).
/// SF Symbols become Segoe MDL2/Fluent glyphs for <c>FontIcon</c> (same bands).
/// </summary>
public static class PlaybackIcons
{
    public const string Repeat = "\uE8EE"; // RepeatAll
    public const string RepeatOne = "\uE8ED"; // RepeatOne
    public const string Shuffle = "\uE8B1";
    public const string Play = "\uE768";
    public const string Pause = "\uE769";
    public const string Previous = "\uE892";
    public const string Next = "\uE893";
    public const string VolumeMute = "\uE74F";
    public const string Volume1 = "\uE993";
    public const string Volume2 = "\uE994";
    public const string Volume3 = "\uE995";

    public static string RepeatIcon(RepeatMode mode) => mode switch
    {
        RepeatMode.One => RepeatOne,
        _ => Repeat,
    };

    public static string Volume(float level)
    {
        if (level <= 0.001f)
        {
            return VolumeMute;
        }
        if (level < 0.33f)
        {
            return Volume1;
        }
        if (level < 0.66f)
        {
            return Volume2;
        }
        return Volume3;
    }
}
