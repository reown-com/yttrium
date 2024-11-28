//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <yttrium_dart/yttrium_dart_plugin.h>

void fl_register_plugins(FlPluginRegistry* registry) {
  g_autoptr(FlPluginRegistrar) yttrium_dart_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "YttriumDartPlugin");
  yttrium_dart_plugin_register_with_registrar(yttrium_dart_registrar);
}
