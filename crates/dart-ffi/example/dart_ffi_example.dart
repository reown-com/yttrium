import 'package:dart_ffi/dart_ffi.dart';

void main() async {
  // var awesome = Awesome();
  // print('awesome: ${awesome.isAwesome}');
  await RustLib.init();
}
