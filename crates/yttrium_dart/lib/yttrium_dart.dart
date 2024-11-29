import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:yttrium_dart/generated/frb_generated.dart' as frb;

import 'yttrium_dart_platform_interface.dart';

class YttriumDart {
  Future<String?> getPlatformVersion() {
    return YttriumDartPlatform.instance.getPlatformVersion();
  }

  Future<void> init() async {
    try {
      // Locate the native library in the app's bundle
      final yttrium =
          ExternalLibrary.open('generated/libyttrium_dart_universal.a');

      // Initialize the Rust library
      await frb.YttriumDart.init(externalLibrary: yttrium);
    } catch (e) {
      print(e.toString());
    }
  }
}
