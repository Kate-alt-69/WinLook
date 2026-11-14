cmake_minimum_required(VERSION 3.16)
project(wl_Lwlde VERSION 0.1.0 LANGUAGES C)

include(GNUInstallDirs)
include(CTest)

add_library(wl_Lwlde STATIC src/wl_Lwlde.c)
add_library(WinLook::Lwlde ALIAS wl_Lwlde)
target_include_directories(wl_Lwlde PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)
target_link_libraries(wl_Lwlde PUBLIC wl_Lanimate wl_Lcpuv wl_Lgpuv)
set_target_properties(wl_Lwlde PROPERTIES
    C_STANDARD 90
    C_STANDARD_REQUIRED YES
    C_EXTENSIONS NO
    POSITION_INDEPENDENT_CODE YES
)

install(TARGETS wl_Lwlde ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})
install(FILES include/wl_Lwlde.h DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

if(BUILD_TESTING)
    add_executable(test_wl_Lwlde tests/test_wl_Lwlde.c)
    target_link_libraries(test_wl_Lwlde PRIVATE wl_Lwlde)
    set_target_properties(test_wl_Lwlde PROPERTIES C_STANDARD 90 C_STANDARD_REQUIRED YES C_EXTENSIONS NO)
    add_test(NAME wl_Lwlde.core COMMAND test_wl_Lwlde)
endif()
