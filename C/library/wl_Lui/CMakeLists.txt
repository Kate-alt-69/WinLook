cmake_minimum_required(VERSION 3.16)
project(wl_Lui VERSION 0.2.0 LANGUAGES C)

include(GNUInstallDirs)

add_library(wl_Lui STATIC
    src/wl_Lui.c
    src/wl_Lui_win32.c
)
add_library(WinLook::Lui ALIAS wl_Lui)

target_include_directories(wl_Lui
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

if(WIN32)
    target_link_libraries(wl_Lui PUBLIC user32 gdi32)
endif()

set_target_properties(wl_Lui PROPERTIES
    C_STANDARD 90
    C_STANDARD_REQUIRED YES
    C_EXTENSIONS NO
    POSITION_INDEPENDENT_CODE YES
)

install(TARGETS wl_Lui
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
)
install(FILES include/wl_Lui.h
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

include(CTest)
if(BUILD_TESTING)
    add_executable(test_wl_Lui tests/test_wl_Lui.c)
    target_link_libraries(test_wl_Lui PRIVATE wl_Lui)
    set_target_properties(test_wl_Lui PROPERTIES
        C_STANDARD 90
        C_STANDARD_REQUIRED YES
        C_EXTENSIONS NO
    )
    if(MSVC)
        target_compile_options(test_wl_Lui PRIVATE /UNDEBUG)
    else()
        target_compile_options(test_wl_Lui PRIVATE -UNDEBUG)
    endif()
    add_test(NAME wl_Lui.core COMMAND test_wl_Lui)
endif()
