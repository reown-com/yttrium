#include "include/yttrium_dart/yttrium_dart_plugin_c_api.h"

#include <flutter/plugin_registrar_windows.h>

#include "yttrium_dart_plugin.h"

void YttriumDartPluginCApiRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  yttrium_dart::YttriumDartPlugin::RegisterWithRegistrar(
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar));
}
