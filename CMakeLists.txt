cmake_minimum_required(VERSION 3.8)
project(sds011)

set(CMAKE_CXX_STANDARD 11)

set(SOURCE_FILES
        src/lib.rs
        src/bin/sds011.rs
        Cargo.toml)

add_executable(sds011 ${SOURCE_FILES})