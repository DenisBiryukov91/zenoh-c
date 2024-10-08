message(STATUS "zenoh-c install")

#
# Installation
# For debug configuration installs libraries with 'd' added to filename
#
status_print(CMAKE_INSTALL_PREFIX)

include(GNUInstallDirs)
include(CMakePackageConfigHelpers)

function(install_zenohc_lib configurations property_postfix package_name)
    if (BUILD_SHARED_LIBS)
        get_target_property(dylib_path zenohc::lib IMPORTED_LOCATION_${property_postfix})
        get_target_property_if_set(implib_path zenohc::lib IMPORTED_IMPLIB_${property_postfix})
        get_filename_component(DYLIB ${dylib_path} NAME)
        get_filename_component(IMPLIB "${implib_path}" NAME)
        # On Windows .dll need to be installed in ${CMAKE_INSTALL_BINDIR}, 
        # while on Linux and macOS .so and .dylib need to be installed in ${CMAKE_INSTALL_LIBDIR}
        if(WIN32)
            set(ZENOHC_INSTALL_DYLIBDIR ${CMAKE_INSTALL_BINDIR})
        else()
            set(ZENOHC_INSTALL_DYLIBDIR ${CMAKE_INSTALL_LIBDIR})
        endif()
        install(FILES ${dylib_path} DESTINATION ${ZENOHC_INSTALL_DYLIBDIR} CONFIGURATIONS ${configurations})
        if(DEFINED implib_path)
            install(FILES ${implib_path} DESTINATION ${CMAKE_INSTALL_LIBDIR} CONFIGURATIONS ${configurations})
        endif()
        if((APPLE OR UNIX))
            get_filename_component(LIBNAME ${DYLIB} NAME_WE)
            configure_file(${CMAKE_CURRENT_SOURCE_DIR}/zenohc.pc.in ${CMAKE_CURRENT_BINARY_DIR}/${package_name}_${property_postfix}.pc @ONLY)
            install(FILES ${CMAKE_CURRENT_BINARY_DIR}/${package_name}_${property_postfix}.pc 
                DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig" 
                CONFIGURATIONS ${configurations}
                RENAME ${package_name}.pc
                OPTIONAL)
        endif()
    else()
        get_target_property(staticlib_path zenohc::static IMPORTED_LOCATION_${property_postfix})
        get_target_property(NATIVE_STATIC_LIBS zenohc::static INTERFACE_LINK_LIBRARIES)
        get_filename_component(STATICLIB ${staticlib_path} NAME)
        install(FILES ${staticlib_path} DESTINATION ${CMAKE_INSTALL_LIBDIR} CONFIGURATIONS ${configurations})
    endif()

    set(CMAKE_INSTALL_CMAKEDIR "${CMAKE_INSTALL_LIBDIR}/cmake/${package_name}")

    # Generate <Package>Config.cmake
    configure_package_config_file(
        "PackageConfig.cmake.in"
        "${CMAKE_CURRENT_BINARY_DIR}/${package_name}_${property_postfix}Config.cmake"
        INSTALL_DESTINATION "${CMAKE_INSTALL_CMAKEDIR}")

    # Generate <Package>Version.cmake
    write_basic_package_version_file(
        "${CMAKE_CURRENT_BINARY_DIR}/${package_name}_${property_postfix}ConfigVersion.cmake"
        VERSION ${PROJECT_VERSION}
        COMPATIBILITY SameMajorVersion)

    install(
        FILES "${CMAKE_CURRENT_BINARY_DIR}/${package_name}_${property_postfix}Config.cmake"
        DESTINATION "${CMAKE_INSTALL_CMAKEDIR}"
        CONFIGURATIONS ${configurations}
        RENAME ${package_name}Config.cmake
        COMPONENT dev)

    install(
        FILES "${CMAKE_CURRENT_BINARY_DIR}/${package_name}_${property_postfix}ConfigVersion.cmake"
        DESTINATION "${CMAKE_INSTALL_CMAKEDIR}"
        CONFIGURATIONS ${configurations}
        RENAME ${package_name}ConfigVersion.cmake
        COMPONENT dev)


endfunction()


#
# Install headers
#
get_target_property(include_dirs zenohc::lib INTERFACE_INCLUDE_DIRECTORIES)
foreach(dir ${include_dirs})
    install(DIRECTORY "${dir}/" DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
endforeach()

#
# Install lib files 
#
install_zenohc_lib("Release;RelWithDebInfo;MinSizeRel;None" "RELEASE" zenohc)
install_zenohc_lib("Debug" "DEBUG" zenohc)
