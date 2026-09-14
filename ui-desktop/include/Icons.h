#pragma once
#include <string>
#include <wx/bmpbndl.h>
#include <wx/gdicmn.h>

namespace Icons {
    constexpr const char* PLAY_CIRCLE = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-miterlimit="10" stroke-width="16"/><polygon points="172 128 108 88 108 168 172 128" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    constexpr const char* PAUSE_CIRCLE = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-miterlimit="10" stroke-width="16"/><line x1="104" y1="96" x2="104" y2="160" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="152" y1="96" x2="152" y2="160" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    constexpr const char* SKIP_BACK_CIRCLE = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-miterlimit="10" stroke-width="16"/><polygon points="96 128 160 88 160 168 96 128" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="96" y1="88" x2="96" y2="168" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    constexpr const char* SKIP_FORWARD_CIRCLE = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><circle cx="128" cy="128" r="96" fill="none" stroke="currentColor" stroke-miterlimit="10" stroke-width="16"/><polygon points="160 128 96 88 96 168 160 128" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="160" y1="88" x2="160" y2="168" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    constexpr const char* REPEAT = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><polyline points="200 88 224 64 200 40" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M32,128A64,64,0,0,1,96,64H224" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="56 168 32 192 56 216" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M224,128a64,64,0,0,1-64,64H32" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    constexpr const char* REPEAT_ONCE = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><polyline points="200 88 224 64 200 40" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M32,128A64,64,0,0,1,96,64H224" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="56 168 32 192 56 216" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M224,128a64,64,0,0,1-64,64H32" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="120 111.99 136 104 136 152" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    constexpr const char* SHUFFLE = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><path d="M32,72H55.06a64,64,0,0,1,52.08,26.8l41.72,58.4A64,64,0,0,0,200.94,184H232" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="208 48 232 72 208 96" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><polyline points="208 160 232 184 208 208" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M147.66,100.47l1.2-1.67A64,64,0,0,1,200.94,72H232" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M32,184H55.06a64,64,0,0,0,52.08-26.8l1.2-1.67" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    constexpr const char* SPEAKER_HIGH = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><path d="M80,168H32a8,8,0,0,1-8-8V96a8,8,0,0,1,8-8H80l72-56V224Z" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="80" y1="88" x2="80" y2="168" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M192,106.85a32,32,0,0,1,0,42.3" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><path d="M221.67,80a72,72,0,0,1,0,96" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    constexpr const char* SPEAKER_X = 
        R"(<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><path d="M80,168H32a8,8,0,0,1-8-8V96a8,8,0,0,1,8-8H80l72-56V224Z" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="240" y1="104" x2="192" y2="152" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="240" y1="152" x2="192" y2="104" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/><line x1="80" y1="88" x2="80" y2="168" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="16"/></svg>)";

    inline wxBitmapBundle CreateSVGIcon(const std::string& svg, const std::string& color, const wxSize& size) {
        std::string colored = svg;
        std::string from = "currentColor";
        size_t pos = 0;
        while ((pos = colored.find(from, pos)) != std::string::npos) {
            colored.replace(pos, from.length(), color);
            pos += color.length();
        }
        return wxBitmapBundle::FromSVG(colored.c_str(), size);
    }
}
