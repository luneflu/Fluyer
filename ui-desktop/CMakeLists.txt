cmake_minimum_required(VERSION 3.20)
project(FluyerDesktop CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(wxWidgets REQUIRED COMPONENTS core base)
include(${wxWidgets_USE_FILE})

set(ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/..)

include_directories(
    include
    ${ROOT_DIR}/crates/fluyer_core/include
)

set(SOURCES
    src/main.cpp
    src/MainFrame.cpp
    src/AlbumListCtrl.cpp
    src/MusicListCtrl.cpp
)

add_executable(FluyerApp ${SOURCES})

target_link_libraries(FluyerApp
    ${wxWidgets_LIBRARIES}
    ${ROOT_DIR}/target/debug/libfluyer_core.dylib
    ${ROOT_DIR}/libs/macos/libbass.dylib
    ${ROOT_DIR}/libs/macos/libbassmix.dylib
)

# Ensure runtime finds dylibs
set_target_properties(FluyerApp PROPERTIES
    BUILD_WITH_INSTALL_RPATH TRUE
    INSTALL_RPATH "@executable_path/../../../../libs/macos;@executable_path/../../../../target/debug"
)
