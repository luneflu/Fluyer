#ifdef __WXOSX__

#import <Cocoa/Cocoa.h>
#import <QuartzCore/QuartzCore.h>
#import <objc/runtime.h>
#include "MainFrameNative.h"
#include <dispatch/dispatch.h>

@interface FluyerBackgroundLayer : CALayer
@property (nonatomic, assign) CGImageRef currentCGImage;
@property (nonatomic, assign) CGImageRef nextCGImage;
@end

@implementation FluyerBackgroundLayer

- (instancetype)init {
    if (self = [super init]) {
        self.currentCGImage = nullptr;
        self.nextCGImage = nullptr;
        self.contentsGravity = kCAGravityResizeAspectFill;
        self.minificationFilter = kCAFilterLinear;
        self.magnificationFilter = kCAFilterLinear;
    }
    return self;
}

- (void)dealloc {
    if (self.currentCGImage) {
        CGImageRelease(self.currentCGImage);
        self.currentCGImage = nullptr;
    }
    if (self.nextCGImage) {
        CGImageRelease(self.nextCGImage);
        self.nextCGImage = nullptr;
    }
}

- (void)setBackgroundCGImage:(CGImageRef)newImage animated:(BOOL)animated {
    if (!newImage) return;

    CGImageRetain(newImage);

    if (!self.currentCGImage) {
        // First image: fade in from transparent
        [self removeAnimationForKey:@"crossFade"];
        self.contents = (__bridge id)newImage;
        if (animated) {
            [CATransaction begin];
            [CATransaction setDisableActions:YES];
            self.opacity = 0.0;
            [CATransaction commit];

            CABasicAnimation* fadeIn = [CABasicAnimation animationWithKeyPath:@"opacity"];
            fadeIn.fromValue = @0.0;
            fadeIn.toValue = @1.0;
            fadeIn.duration = 0.75;
            fadeIn.fillMode = kCAFillModeForwards;
            fadeIn.removedOnCompletion = NO;
            fadeIn.timingFunction = [CAMediaTimingFunction functionWithName:kCAMediaTimingFunctionEaseInEaseOut];
            [self addAnimation:fadeIn forKey:@"fadeIn"];

            dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(0.75 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
                [CATransaction begin];
                [CATransaction setDisableActions:YES];
                self.opacity = 1.0;
                [CATransaction commit];
                [self removeAnimationForKey:@"fadeIn"];
            });
        }
        if (self.currentCGImage) CGImageRelease(self.currentCGImage);
        self.currentCGImage = CGImageRetain(newImage);
    } else {
        // Interrupt any running animation: snapshot current visual state first
        CALayer* presentation = self.presentationLayer;
        CGImageRef visualCurrent = presentation
            ? (__bridge CGImageRef)presentation.contents
            : self.currentCGImage;

        [self removeAnimationForKey:@"fadeIn"];
        [self removeAnimationForKey:@"crossFade"];

        // Set layer contents to the visual current frame without animation
        [CATransaction begin];
        [CATransaction setDisableActions:YES];
        self.contents = (__bridge id)(visualCurrent ? visualCurrent : self.currentCGImage);
        self.opacity = 1.0;
        [CATransaction commit];

        if (animated) {
            CABasicAnimation* crossFade = [CABasicAnimation animationWithKeyPath:@"contents"];
            crossFade.fromValue = self.contents; // actual visible frame
            crossFade.toValue = (__bridge id)newImage;
            crossFade.duration = 0.75;
            crossFade.fillMode = kCAFillModeForwards;
            crossFade.removedOnCompletion = NO;
            crossFade.timingFunction = [CAMediaTimingFunction functionWithName:kCAMediaTimingFunctionEaseInEaseOut];
            [self addAnimation:crossFade forKey:@"crossFade"];

            CGImageRef capturedNew = CGImageRetain(newImage);
            dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(0.75 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
                [CATransaction begin];
                [CATransaction setDisableActions:YES];
                self.contents = (__bridge id)capturedNew;
                [CATransaction commit];
                [self removeAnimationForKey:@"crossFade"];
                CGImageRelease(capturedNew);
            });
        } else {
            [CATransaction begin];
            [CATransaction setDisableActions:YES];
            self.contents = (__bridge id)newImage;
            [CATransaction commit];
        }

        if (self.currentCGImage) CGImageRelease(self.currentCGImage);
        self.currentCGImage = CGImageRetain(newImage);
    }

    if (self.nextCGImage) CGImageRelease(self.nextCGImage);
    self.nextCGImage = newImage; // already retained above
}

@end

static const char* kFluyerBackgroundLayerKey = "FluyerBackgroundLayer";

void MainFrameNative::InitializeBackgroundLayer(void* nsView) {
    if (!nsView) return;
    
    NSView* view = (__bridge NSView*)nsView;
    dispatch_async(dispatch_get_main_queue(), ^{
        view.wantsLayer = YES;
        
        FluyerBackgroundLayer* bgLayer = [[FluyerBackgroundLayer alloc] init];
        bgLayer.frame = view.bounds;
        bgLayer.autoresizingMask = kCALayerWidthSizable | kCALayerHeightSizable;
        
        [view.layer insertSublayer:bgLayer atIndex:0];
        
        objc_setAssociatedObject((__bridge id)nsView, kFluyerBackgroundLayerKey, bgLayer, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
    });
}

void MainFrameNative::SetBackgroundImage(void* nsView, uint8_t* rgbaData, uint32_t width, uint32_t height, bool animated) {
    if (!nsView || !rgbaData || width == 0 || height == 0) return;
    
    // Create CGImage from RGBA data
    size_t dataSize = width * height * 4;
    uint8_t* dataCopy = (uint8_t*)malloc(dataSize);
    memcpy(dataCopy, rgbaData, dataSize);
    
    CGDataProviderRef provider = CGDataProviderCreateWithData(
        nullptr,
        dataCopy,
        dataSize,
        [](void* info, const void* data, size_t size) {
            free((void*)data);
        }
    );
    
    CGColorSpaceRef colorSpace = CGColorSpaceCreateDeviceRGB();
    CGImageRef image = CGImageCreate(
        width,
        height,
        8,
        32,
        width * 4,
        colorSpace,
        kCGImageAlphaLast,
        provider,
        nullptr,
        false,
        kCGRenderingIntentDefault
    );
    
    CGColorSpaceRelease(colorSpace);
    CGDataProviderRelease(provider);
    
    if (!image) return;
    
    NSView* view = (__bridge NSView*)nsView;
    dispatch_async(dispatch_get_main_queue(), ^{
        FluyerBackgroundLayer* bgLayer = objc_getAssociatedObject((__bridge id)nsView, kFluyerBackgroundLayerKey);
        if (bgLayer) {
            [bgLayer setBackgroundCGImage:image animated:animated];
        }
        CGImageRelease(image);
    });
}

void MainFrameNative::CleanupBackgroundLayer(void* nsView) {
    if (!nsView) return;
    
    dispatch_async(dispatch_get_main_queue(), ^{
        FluyerBackgroundLayer* bgLayer = objc_getAssociatedObject((__bridge id)nsView, kFluyerBackgroundLayerKey);
        if (bgLayer) {
            [bgLayer removeFromSuperlayer];
            objc_setAssociatedObject((__bridge id)nsView, kFluyerBackgroundLayerKey, nil, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
        }
    });
}

#endif // __WXOSX__
