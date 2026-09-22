#pragma once
#include <string>
#include <wx/bmpbndl.h>
#include <wx/colour.h>
#include <wx/gdicmn.h>
#include <wx/settings.h>
#include <wx/string.h>

#if defined(__WXOSX__) || defined(__APPLE__)
#import <AppKit/AppKit.h>
#include <algorithm>
#include <cmath>
#endif

namespace Icons {

enum Icon {
  Play,
  Pause,
  Prev,
  Next,
  Repeat,
  RepeatOnce,
  Shuffle,
  SpeakerHigh,
  SpeakerMute
};

#if defined(__WXOSX__) || defined(__APPLE__)
// ponytail: native SF Symbol loader for macOS; SVG strings omitted to save
// binary size.
inline wxBitmapBundle CreateNativeIcon(const std::string &name,
                                       const wxColour &color,
                                       const wxSize &size) {
  @autoreleasepool {
    NSString *symName = [NSString stringWithUTF8String:name.c_str()];
    NSImage *sym = [NSImage imageWithSystemSymbolName:symName
                             accessibilityDescription:nil];
    if (!sym) {
      sym = [NSImage imageNamed:symName];
    }
    if (!sym)
      return wxBitmapBundle();

    double pt = std::floor(std::min(size.GetWidth(), size.GetHeight()) * 0.62);
    wxColour col = color.IsOk()
                       ? color
                       : wxSystemSettings::GetColour(wxSYS_COLOUR_BTNTEXT);
    NSColor *nsCol = [NSColor colorWithSRGBRed:col.Red() / 255.0
                                         green:col.Green() / 255.0
                                          blue:col.Blue() / 255.0
                                         alpha:col.Alpha() / 255.0];
    NSImageSymbolConfiguration *sizeConfig = [NSImageSymbolConfiguration
        configurationWithPointSize:pt
                            weight:NSFontWeightMedium];
    NSImageSymbolConfiguration *colConfig =
        [NSImageSymbolConfiguration configurationWithPaletteColors:@[ nsCol ]];
    NSImageSymbolConfiguration *config =
        [sizeConfig configurationByApplyingConfiguration:colConfig];
    NSImage *configured = [sym imageWithSymbolConfiguration:config];
    if (!configured)
      configured = sym;

    NSSize targetNSSize = NSMakeSize(size.GetWidth(), size.GetHeight());
    NSImage *canvas = [NSImage
         imageWithSize:targetNSSize
               flipped:NO
        drawingHandler:^BOOL(NSRect dstRect) {
          NSSize symSize = [configured size];
          CGFloat w = std::min(symSize.width, dstRect.size.width);
          CGFloat h = std::min(symSize.height, dstRect.size.height);
          NSRect r =
              NSMakeRect(std::round((dstRect.size.width - w) / 2.0),
                         std::round((dstRect.size.height - h) / 2.0), w, h);
          [configured drawInRect:r
                        fromRect:NSZeroRect
                       operation:NSCompositingOperationSourceOver
                        fraction:1.0];
          return YES;
        }];
    [canvas setTemplate:NO];

    wxBitmap bmp((WXImage)canvas);
    return wxBitmapBundle::FromBitmap(bmp);
  }
}

inline wxBitmapBundle Get(Icon icon, const wxColour &color,
                          const wxSize &size) {
  const char *name = "";
  switch (icon) {
  case Icon::Play:
    name = "play.fill";
    break;
  case Icon::Pause:
    name = "pause.fill";
    break;
  case Icon::Prev:
    name = "backward.end.fill";
    break;
  case Icon::Next:
    name = "forward.end.fill";
    break;
  case Icon::Repeat:
    name = "repeat";
    break;
  case Icon::RepeatOnce:
    name = "repeat.1";
    break;
  case Icon::Shuffle:
    name = "shuffle";
    break;
  case Icon::SpeakerHigh:
    name = "speaker.wave.2.fill";
    break;
  case Icon::SpeakerMute:
    name = "speaker.slash.fill";
    break;
  }
  return CreateNativeIcon(name, color, size);
}

#else

constexpr const char *PLAY_CIRCLE =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-miterlimit="10" stroke-width="16"/><polygon points="172 128 108 88 108 168 172 128" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

constexpr const char *PAUSE_CIRCLE =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-miterlimit="10" stroke-width="16"/><line x1="104" y1="96" x2="104" y2="160" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="152" y1="96" x2="152" y2="160" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

constexpr const char *SKIP_BACK_CIRCLE =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-miterlimit="10" stroke-width="16"/><polygon points="96 128 160 88 160 168 96 128" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="96" y1="88" x2="96" y2="168" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

constexpr const char *SKIP_FORWARD_CIRCLE =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-miterlimit="10" stroke-width="16"/><polygon points="160 128 96 88 96 168 160 128" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="160" y1="88" x2="160" y2="168" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

constexpr const char *REPEAT =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><polyline points="200 88 224 64 200 40" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M32,128A64,64,0,0,1,96,64H224" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="56 168 32 192 56 216" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M224,128a64,64,0,0,1-64,64H32" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

constexpr const char *REPEAT_ONCE =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><polyline points="200 88 224 64 200 40" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M32,128A64,64,0,0,1,96,64H224" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="56 168 32 192 56 216" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M224,128a64,64,0,0,1-64,64H32" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="120 111.99 136 104 136 152" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

constexpr const char *SHUFFLE =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><path d="M32,72H55.06a64,64,0,0,1,52.08,26.8l41.72,58.4A64,64,0,0,0,200.94,184H232" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="208 48 232 72 208 96" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="208 160 232 184 208 208" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M147.66,100.47l1.2-1.67A64,64,0,0,1,200.94,72H232" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M32,184H55.06a64,64,0,0,0,52.08-26.8l1.2-1.67" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

constexpr const char *SPEAKER_HIGH =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><path d="M80,168H32a8,8,0,0,1-8-8V96a8,8,0,0,1,8-8H80l72-56V224Z" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="80" y1="88" x2="80" y2="168" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M192,106.85a32,32,0,0,1,0,42.3" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M221.67,80a72,72,0,0,1,0,96" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

constexpr const char *SPEAKER_X =
    R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><path d="M80,168H32a8,8,0,0,1-8-8V96a8,8,0,0,1,8-8H80l72-56V224Z" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="240" y1="104" x2="192" y2="152" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="240" y1="152" x2="192" y2="104" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="80" y1="88" x2="80" y2="168" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

inline wxBitmapBundle CreateSVGIcon(const std::string &svg,
                                    const wxColour &color, const wxSize &size) {
  std::string hex = wxString::Format("#%02X%02X%02X", color.Red(),
                                     color.Green(), color.Blue())
                        .ToStdString();
  std::string colored = svg;
  std::string from = "currentColor";
  size_t pos = 0;
  while ((pos = colored.find(from, pos)) != std::string::npos) {
    colored.replace(pos, from.length(), hex);
    pos += hex.length();
  }
  return wxBitmapBundle::FromSVG(colored.c_str(), size);
}

inline wxBitmapBundle Get(Icon icon, const wxColour &color,
                          const wxSize &size) {
  const char *svg = "";
  switch (icon) {
  case Icon::Play:
    svg = PLAY_CIRCLE;
    break;
  case Icon::Pause:
    svg = PAUSE_CIRCLE;
    break;
  case Icon::Prev:
    svg = SKIP_BACK_CIRCLE;
    break;
  case Icon::Next:
    svg = SKIP_FORWARD_CIRCLE;
    break;
  case Icon::Repeat:
    svg = REPEAT;
    break;
  case Icon::RepeatOnce:
    svg = REPEAT_ONCE;
    break;
  case Icon::Shuffle:
    svg = SHUFFLE;
    break;
  case Icon::SpeakerHigh:
    svg = SPEAKER_HIGH;
    break;
  case Icon::SpeakerMute:
    svg = SPEAKER_X;
    break;
  }
  return CreateSVGIcon(svg, color, size);
}

#endif

} // namespace Icons
