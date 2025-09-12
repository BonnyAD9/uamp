cmake_minimum_required(VERSION 3.16)

project(uamp-ffmpeg VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_library(uamp-ffmpeg SHARED)

file(GLOB uamp-ffmpeg_files CONFIGURE_DEPENDS "src/**/*.cpp" "src/*.cpp")
target_sources(uamp-ffmpeg PRIVATE ${uamp-ffmpeg_files})

target_link_libraries(uamp-ffmpeg PRIVATE avformat avcodec avutil)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -Werror")
endif()
