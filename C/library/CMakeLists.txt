cmake_minimum_required(VERSION 3.16)
project(WinLookSharedLibraries LANGUAGES C)

include(CTest)
add_subdirectory(wl_Lanimate)
add_subdirectory(wl_Lcpuv)
add_subdirectory(wl_Lgpuv)
add_subdirectory(wl_Lui)
