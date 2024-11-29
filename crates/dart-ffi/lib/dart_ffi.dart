/// Support for doing something awesome.
///
/// More dartdocs go here.
library;

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:dart_ffi/generated/frb_generated.dart';
import 'package:dylib/dylib.dart';

export 'generated/lib.dart';

class DartFfi {
  static Future<void> init() async {
    // Locate the native library file
    final dylibPath = resolveDylibPath('libyttrium_dart', path: 'assets');
    final yttrium = ExternalLibrary.open(dylibPath);
    // Initialize the Rust library
    await YttriumDart.init(externalLibrary: yttrium);
  }
}
