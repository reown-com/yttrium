import 'package:dart_ffi/dart_ffi.dart';

void main() async {
  // Locate the native library file
  final externalLibrary = ExternalLibrary.open(
    '../../target/debug/libdart_yttrium.dylib',
  );

  // Initialize the Rust library
  await YttriumDart.init(externalLibrary: externalLibrary);

  // final config = AccountClientConfigI(
  //   chainId: BigInt.from(1),
  //   config: ConfigI(),
  //   safe: false,
  //   signerType: '',
  // );
  // final accountClient = await AccountClient.newInstance(config: config);

  // final api = (RustLib.instance.api as RustLibApiImpl);

  final chainAbstraction = await ChainAbstractionClient.newInstance(
    projectId: 'cad4956f31a5e40a00b62865b030c6f8',
  );
  final fees = await chainAbstraction.estimateFees(chainId: 'eip155:1');
  print(fees.maxFeePerGas);
  print(fees.maxPriorityFeePerGas);

  final _ = Eip1559Estimation(
    maxFeePerGas: fees.maxFeePerGas,
    maxPriorityFeePerGas: fees.maxPriorityFeePerGas,
  );
}

// class AccountClientConfigI implements AccountClientConfig {
//   AccountClientConfigI({
//     required this.chainId,
//     required this.config,
//     required this.ownerAddress,
//     required this.privateKey,
//     required this.safe,
//     required this.signerType,
//   });

//   @override
//   BigInt chainId;

//   @override
//   Config config;

//   @override
//   String ownerAddress;

//   @override
//   String privateKey;

//   @override
//   bool safe;

//   @override
//   String signerType;

//   @override
//   void dispose() {
//     // TODO: implement dispose
//   }

//   @override
//   // TODO: implement isDisposed
//   bool get isDisposed => throw UnimplementedError();
// }

// class ConfigI implements Config {
//   @override
//   void dispose() {
//     // TODO: implement dispose
//   }

//   @override
//   // TODO: implement isDisposed
//   bool get isDisposed => throw UnimplementedError();
// }
