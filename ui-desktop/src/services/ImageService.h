#pragma once
#include <unordered_map>
#include <wx/bitmap.h>
#include <wx/image.h>
#include <wx/mstream.h>
#include "fluyer_core.h"

class ImageService {
public:
    explicit ImageService(FluyerEngine* engine = nullptr);

    void SetEngine(FluyerEngine* engine);

    wxBitmap GetTrackImage(uintptr_t index, int targetSize, double scaleFactor);
    wxBitmap GetAlbumImage(uintptr_t index, int targetSize, double scaleFactor);
    wxBitmap GetCurrentImage(int targetSize, double scaleFactor);

    void InvalidateTrackCover(uintptr_t index);
    void InvalidateAlbumCover(uintptr_t index);
    void ClearCache();

private:
    static wxBitmap LoadBitmapFromBytes(uint8_t* bytes, uintptr_t len, int targetSize, double scaleFactor);

    FluyerEngine* m_engine = nullptr;
    std::unordered_map<uintptr_t, wxBitmap> m_trackImageCache;
    std::unordered_map<uintptr_t, wxBitmap> m_albumImageCache;
};
