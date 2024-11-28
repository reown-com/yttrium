import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'yttrium_dart_method_channel.dart';

abstract class YttriumDartPlatform extends PlatformInterface {
  /// Constructs a YttriumDartPlatform.
  YttriumDartPlatform() : super(token: _token);

  static final Object _token = Object();

  static YttriumDartPlatform _instance = MethodChannelYttriumDart();

  /// The default instance of [YttriumDartPlatform] to use.
  ///
  /// Defaults to [MethodChannelYttriumDart].
  static YttriumDartPlatform get instance => _instance;

  /// Platform-specific implementations should set this with their own
  /// platform-specific class that extends [YttriumDartPlatform] when
  /// they register themselves.
  static set instance(YttriumDartPlatform instance) {
    PlatformInterface.verifyToken(instance, _token);
    _instance = instance;
  }

  Future<String?> getPlatformVersion() {
    throw UnimplementedError('platformVersion() has not been implemented.');
  }
}
