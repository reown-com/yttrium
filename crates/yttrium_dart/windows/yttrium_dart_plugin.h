#ifndef FLUTTER_PLUGIN_YTTRIUM_DART_PLUGIN_H_
#define FLUTTER_PLUGIN_YTTRIUM_DART_PLUGIN_H_

#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>

#include <memory>

namespace yttrium_dart {

class YttriumDartPlugin : public flutter::Plugin {
 public:
  static void RegisterWithRegistrar(flutter::PluginRegistrarWindows *registrar);

  YttriumDartPlugin();

  virtual ~YttriumDartPlugin();

  // Disallow copy and assign.
  YttriumDartPlugin(const YttriumDartPlugin&) = delete;
  YttriumDartPlugin& operator=(const YttriumDartPlugin&) = delete;

  // Called when a method is called on this plugin's channel from Dart.
  void HandleMethodCall(
      const flutter::MethodCall<flutter::EncodableValue> &method_call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result);
};

}  // namespace yttrium_dart

#endif  // FLUTTER_PLUGIN_YTTRIUM_DART_PLUGIN_H_
