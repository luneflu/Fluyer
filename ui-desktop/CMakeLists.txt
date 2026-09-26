cmake_minimum_required(VERSION 3.20)
project(FluyerDesktop CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(wxWidgets REQUIRED COMPONENTS core base)
include(${wxWidgets_USE_FILE})

include(FetchContent)
FetchContent_Declare(
    json
    URL https://github.com/nlohmann/json/releases/download/v3.11.3/json.tar.xz
)
FetchContent_MakeAvailable(json)

set(ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/..)

include_directories(
    src
    ${ROOT_DIR}/crates/fluyer_core/include
)

set(SOURCES
    src/main.cpp
    src/services/LibraryService.cpp
    src/services/PlayerService.cpp
    src/services/ImageService.cpp
    src/ui/MainFrame.cpp
    src/ui/AlbumListCtrl.cpp
    src/ui/CollectionInfoCtrl.cpp
    src/ui/MusicListCtrl.cpp
    src/ui/PlayerBarCtrl.cpp
    src/ui/PlayViewCtrl.cpp
)

if(APPLE)
    list(APPEND SOURCES src/ui/MainFrameNative.mm)
    set_source_files_properties(
        src/ui/PlayerBarCtrl.cpp
        src/ui/PlayViewCtrl.cpp
        src/ui/CollectionInfoCtrl.cpp
        src/ui/MainFrameNative.mm
        PROPERTIES COMPILE_FLAGS "-x objective-c++ -fobjc-arc"
    )
endif()

add_executable(FluyerApp ${SOURCES})

target_link_libraries(FluyerApp
    ${wxWidgets_LIBRARIES}
    nlohmann_json::nlohmann_json
    ${ROOT_DIR}/target/debug/libfluyer_core.dylib
    ${ROOT_DIR}/libs/macos/libbass.dylib
    ${ROOT_DIR}/libs/macos/libbassmix.dylib
)

if(APPLE)
    target_link_libraries(FluyerApp "-framework QuartzCore" "-framework Cocoa")
endif()

# Ensure runtime finds dylibs
set_target_properties(FluyerApp PROPERTIES
    BUILD_WITH_INSTALL_RPATH TRUE
    INSTALL_RPATH "@executable_path/../../libs/macos;@executable_path/../../target/debug;@executable_path/../../../../libs/macos;@executable_path/../../../../target/debug"
)

add_executable(test_lyrics tests/test_lyrics.cpp src/services/PlayerService.cpp)
target_link_libraries(test_lyrics
    nlohmann_json::nlohmann_json
    ${ROOT_DIR}/target/debug/libfluyer_core.dylib
    ${ROOT_DIR}/libs/macos/libbass.dylib
    ${ROOT_DIR}/libs/macos/libbassmix.dylib
)
set_target_properties(test_lyrics PROPERTIES
    BUILD_WITH_INSTALL_RPATH TRUE
    INSTALL_RPATH "@executable_path/../../libs/macos;@executable_path/../../target/debug;@executable_path/../../../../libs/macos;@executable_path/../../../../target/debug"
)
