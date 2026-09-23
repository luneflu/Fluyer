#pragma once

#ifdef __WXOSX__

#include <cstdint>
#include <string>

#ifdef __OBJC__
#import <QuartzCore/QuartzCore.h>
typedef CALayer* FluyerBackgroundLayerRef;
#else
typedef void* FluyerBackgroundLayerRef;
#endif

class MainFrameNative {
public:
    static void InitializeBackgroundLayer(void* nsView);
    static void SetBackgroundImage(void* nsView, uint8_t* rgbaData, uint32_t width, uint32_t height, bool animated);
    static void CleanupBackgroundLayer(void* nsView);
};

#endif // __WXOSX__
