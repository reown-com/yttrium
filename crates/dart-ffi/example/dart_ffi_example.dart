import 'package:dart_ffi/dart_ffi.dart';

void main() async {
  // // Locate the native library file
  // final yttrium = ExternalLibrary.open(
  //   '../../target/release/libyttrium_dart.dylib',
  // );
  // // Initialize the Rust library
  // await YttriumDart.init(externalLibrary: yttrium);

  await DartFfi.init();

  // final config = AccountClientConfigI(
  //   chainId: BigInt.from(1),
  //   config: ConfigI(),
  //   safe: false,
  //   signerType: '',
  //   ownerAddress: '',
  //   privateKey: '',
  // );

  // final accountClient = await AccountClient.newInstance(config: config);

  final chainAbstraction = await ChainAbstractionClient.newInstance(
    projectId: 'cad4956f31a5e40a00b62865b030c6f8',
  );
  final fees = await chainAbstraction.estimateFees(chainId: 'eip155:1');
  print(fees.maxFeePerGas);
  print(fees.maxPriorityFeePerGas);
  // await chainAbstraction.route(transaction: transaction);
  // await chainAbstraction.status(orchestrationId: orchestrationId);
  // await chainAbstraction.waitForSuccessWithTimeout(
  //   orchestrationId: orchestrationId,
  //   checkIn: checkIn,
  //   timeout: timeout,
  // );

  // 1. try to generate code by pointing directly to Yttrium crate
  // 2. Try by using a build.rs file.
  // 3. implement chainabstraction on walletkit side and sample wallet
  // 4. automate the code generation
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
