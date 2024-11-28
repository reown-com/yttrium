import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:yttrium_dart/generated/frb_generated.dart' as frb;

import 'yttrium_dart_platform_interface.dart';

class YttriumDart {
  Future<String?> getPlatformVersion() {
    return YttriumDartPlatform.instance.getPlatformVersion();
  }

  Future<void> init() async {
    // Locate the native library file
    final externalLibrary = ExternalLibrary.open(
      '../../target/debug/libdart_yttrium.dylib',
    );
    // Initialize the Rust library
    await frb.YttriumDart.init(externalLibrary: externalLibrary);
  }
}
