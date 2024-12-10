import Flutter
import UIKit

public class YttriumDartPlugin: NSObject, FlutterPlugin {
  public static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(name: "yttrium_dart", binaryMessenger: registrar.messenger())
    let instance = YttriumDartPlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
  }
}
