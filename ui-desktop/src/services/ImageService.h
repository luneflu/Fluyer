#pragma once
#include <unordered_map>
#include <unordered_set>
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
    
    // Visibility tracking - only keep images currently visible
    void MarkTrackVisible(uintptr_t index);
    void MarkAlbumVisible(uintptr_t index);
    void BeginVisibilityUpdate();
    void EndVisibilityUpdate();

private:
    struct CacheKey {
        uintptr_t index;
        int size;
        double scale;
        
        bool operator==(const CacheKey& other) const {
            return index == other.index && size == other.size && scale == other.scale;
        }
    };
    
    struct CacheKeyHash {
        size_t operator()(const CacheKey& key) const {
            return std::hash<uintptr_t>()(key.index) ^ 
                   (std::hash<int>()(key.size) << 1) ^
                   (std::hash<double>()(key.scale) << 2);
        }
    };

    static wxBitmap LoadBitmapFromBytes(uint8_t* bytes, uintptr_t len, int targetSize, double scaleFactor);

    FluyerEngine* m_engine = nullptr;
    std::unordered_map<CacheKey, wxBitmap, CacheKeyHash> m_trackImageCache;
    std::unordered_map<CacheKey, wxBitmap, CacheKeyHash> m_albumImageCache;
    
    // Track which indices are currently visible
    std::unordered_set<uintptr_t> m_visibleTracks;
    std::unordered_set<uintptr_t> m_visibleAlbums;
    std::unordered_set<uintptr_t> m_newVisibleTracks;
    std::unordered_set<uintptr_t> m_newVisibleAlbums;
};
