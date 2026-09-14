#include "ImageService.h"

ImageService::ImageService(FluyerEngine* engine)
    : m_engine(engine) {}

void ImageService::SetEngine(FluyerEngine* engine) {
    m_engine = engine;
    ClearCache();
}

wxBitmap ImageService::LoadBitmapFromBytes(uint8_t* bytes, uintptr_t len, int targetSize, double scaleFactor) {
    if (!bytes || len == 0) return wxBitmap{};

    wxMemoryInputStream stream(bytes, len);
    wxImage img(stream);
    if (!img.IsOk()) return wxBitmap{};

    int target = static_cast<int>(targetSize * scaleFactor);
    wxImage scaled = img.Scale(target, target, wxIMAGE_QUALITY_HIGH);
    return wxBitmap(scaled, -1, scaleFactor);
}

wxBitmap ImageService::GetTrackImage(uintptr_t index, int targetSize, double scaleFactor) {
    auto it = m_trackImageCache.find(index);
    if (it != m_trackImageCache.end()) {
        return it->second;
    }

    if (!m_engine) return wxBitmap{};

    uintptr_t imgLen = 0;
    uint8_t* bytes = fluyer_library_get_track_image(m_engine, index, &imgLen);
    wxBitmap bmp;
    if (bytes && imgLen > 0) {
        bmp = LoadBitmapFromBytes(bytes, imgLen, targetSize, scaleFactor);
        fluyer_bytes_free(bytes, imgLen);
    }
    m_trackImageCache[index] = bmp;
    return bmp;
}

wxBitmap ImageService::GetAlbumImage(uintptr_t index, int targetSize, double scaleFactor) {
    auto it = m_albumImageCache.find(index);
    if (it != m_albumImageCache.end()) {
        return it->second;
    }

    if (!m_engine) return wxBitmap{};

    uintptr_t imgLen = 0;
    uint8_t* bytes = fluyer_library_get_album_image(m_engine, index, &imgLen);
    wxBitmap bmp;
    if (bytes && imgLen > 0) {
        bmp = LoadBitmapFromBytes(bytes, imgLen, targetSize, scaleFactor);
        fluyer_bytes_free(bytes, imgLen);
    }
    m_albumImageCache[index] = bmp;
    return bmp;
}

wxBitmap ImageService::GetCurrentImage(int targetSize, double scaleFactor) {
    if (!m_engine) return wxBitmap{};

    uintptr_t imgLen = 0;
    uint8_t* bytes = fluyer_player_get_current_image(m_engine, &imgLen);
    wxBitmap bmp;
    if (bytes && imgLen > 0) {
        bmp = LoadBitmapFromBytes(bytes, imgLen, targetSize, scaleFactor);
        fluyer_bytes_free(bytes, imgLen);
    }
    return bmp;
}

void ImageService::InvalidateTrackCover(uintptr_t index) {
    m_trackImageCache.erase(index);
}

void ImageService::InvalidateAlbumCover(uintptr_t index) {
    m_albumImageCache.erase(index);
}

void ImageService::ClearCache() {
    m_trackImageCache.clear();
    m_albumImageCache.clear();
}
