// import 'package:flutter/foundation.dart';
// import 'package:flutter/services.dart';

// import 'yttrium_dart_platform_interface.dart';

// /// An implementation of [YttriumDartPlatform] that uses method channels.
// class MethodChannelYttriumDart extends YttriumDartPlatform {
//   /// The method channel used to interact with the native platform.
//   @visibleForTesting
//   final methodChannel = const MethodChannel('yttrium_dart');

//   @override
//   Future<String?> getPlatformVersion() async {
//     final version = await methodChannel.invokeMethod<String>('getPlatformVersion');
//     return version;
//   }
// }
