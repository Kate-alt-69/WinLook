cmake_minimum_required(VERSION 3.16)
project(wl_Lanimate VERSION 0.1.0 LANGUAGES C)

include(GNUInstallDirs)

add_library(wl_Lanimate STATIC
    src/wl_Lanimate_math.c
    src/wl_Lanimate_tween.c
    src/wl_Lanimate_track.c
    src/wl_Lanimate_animation.c
    src/wl_Lanimate_clock.c
)
add_library(WinLook::Lanimate ALIAS wl_Lanimate)

# The portable clock and animation runtime use fmod/modf on non-MSVC hosts.
# Publish libm as a transitive requirement of the static library.
if(NOT MSVC)
    target_link_libraries(wl_Lanimate PUBLIC m)
endif()

target_include_directories(wl_Lanimate
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

set_target_properties(wl_Lanimate PROPERTIES
    C_STANDARD 90
    C_STANDARD_REQUIRED YES
    C_EXTENSIONS NO
    POSITION_INDEPENDENT_CODE YES
)

install(TARGETS wl_Lanimate
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
)
install(FILES include/wl_Lanimate.h
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

include(CTest)
if(BUILD_TESTING)
    add_executable(test_wl_Lanimate tests/test_wl_Lanimate.c)
    target_link_libraries(test_wl_Lanimate PRIVATE wl_Lanimate)
    set_target_properties(test_wl_Lanimate PROPERTIES
        C_STANDARD 90
        C_STANDARD_REQUIRED YES
        C_EXTENSIONS NO
    )
    if(MSVC)
        target_compile_options(test_wl_Lanimate PRIVATE /UNDEBUG)
    else()
        target_compile_options(test_wl_Lanimate PRIVATE -UNDEBUG)
    endif()
    add_test(NAME wl_Lanimate.core COMMAND test_wl_Lanimate)
endif()
