import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
// import 'package:yttrium_dart/generated/chain_abstraction/api/prepare.dart';
import 'package:yttrium_dart/generated/chain_abstraction/api/status.dart';
// import 'package:yttrium_dart/generated/chain_abstraction/client.dart';
import 'package:yttrium_dart/generated/chain_abstraction/currency.dart';
import 'package:yttrium_dart/generated/chain_abstraction/dart_compat.dart';
import 'package:yttrium_dart/generated/chain_abstraction/error.dart';
import 'package:yttrium_dart/generated/frb_generated.dart' as frb;

abstract class IYttriumClient extends ChainAbstractionClient {
  Future<void> init({required String projectId});
}

class YttriumDart implements IYttriumClient {
  // Private constructor
  YttriumDart._internal();

  // Singleton instance
  static final YttriumDart _instance = YttriumDart._internal();

  // Public accessor for the singleton instance
  static YttriumDart get instance => _instance;

  late final ChainAbstractionClient _chainAbstractionClient;

  @override
  Future<void> init({required String projectId}) async {
    try {
      // Locate the native library file
      final yttrium = Platform.isAndroid
          ? ExternalLibrary.open('libyttrium_dart.so')
          : (Platform.isIOS || Platform.isMacOS)
              ? ExternalLibrary.open('libyttrium_dart_universal.dylib')
              : throw 'Yttrium not yet supported on ${Platform.localeName}';
      // Initialize the Rust library
      await frb.YttriumDart.init(externalLibrary: yttrium);
      // Create ChainAbstractionClient instance
      _chainAbstractionClient = await ChainAbstractionClient.newInstance(
        projectId: projectId,
        pulseMetadata: PulseMetadataCompat(
          url: 'https://appkit-lab.reown.com/flutter_appkit',
          bundleId: 'com.walletconnect.flutterdapp.internal',
          packageName: 'com.walletconnect.flutterdapp.internal',
          sdkVersion: '1.0.0',
          sdkPlatform: Platform.operatingSystem,
        ),
      );
    } catch (e) {
      rethrow;
    }
  }

  @override
  Future<Eip1559EstimationCompat> estimateFees({
    required String chainId,
  }) async {
    return await _chainAbstractionClient.estimateFees(
      chainId: chainId,
    );
  }

  @override
  Future<String> erc20TokenBalance({
    required String chainId,
    required String token,
    required String owner,
  }) async {
    return await _chainAbstractionClient.erc20TokenBalance(
      chainId: chainId,
      token: token,
      owner: owner,
    );
  }

  @override
  Future<PrepareDetailedResponse> prepareDetailed({
    required String chainId,
    required String from,
    required CallCompat call,
    required Currency localCurrency,
  }) async {
    return await _chainAbstractionClient.prepareDetailed(
      chainId: chainId,
      from: from,
      call: call,
      localCurrency: localCurrency,
    );
  }

  // @override
  // Future<FFICall> prepareErc20TransferCall({
  //   required String erc20Address,
  //   required String to,
  //   required BigInt amount,
  // }) async {
  //   if (_chainAbstractionClient == null) {
  //     throw 'ChainAbstractionClient is not initialized';
  //   }
  //   return await _chainAbstractionClient!.prepareErc20TransferCall(
  //     erc20Address: erc20Address,
  //     to: to,
  //     amount: amount,
  //   );
  // }

  // @override
  // Future<UiFieldsCompat> getUiFields({
  //   required PrepareResponseAvailable routeResponse,
  //   required Currency currency,
  // }) async {
  //   return await _chainAbstractionClient.getUiFields(
  //     routeResponse: routeResponse,
  //     currency: currency,
  //   );
  // }

  @override
  Future<StatusResponse> status({required String orchestrationId}) async {
    return await _chainAbstractionClient.status(
      orchestrationId: orchestrationId,
    );
  }

  // @override
  // Future<ExecuteDetails> execute({
  //   required UiFieldsCompat uiFields,
  //   required List<PrimitiveSignatureCompat> routeTxnSigs,
  //   required PrimitiveSignatureCompat initialTxnSig,
  // }) async {
  //   return await _chainAbstractionClient.execute(
  //     uiFields: uiFields,
  //     routeTxnSigs: routeTxnSigs,
  //     initialTxnSig: initialTxnSig,
  //   );
  // }

  @override
  Future<StatusResponseCompleted> waitForSuccessWithTimeout({
    required String orchestrationId,
    required BigInt checkIn,
    required BigInt timeout,
  }) async {
    return await _chainAbstractionClient.waitForSuccessWithTimeout(
      orchestrationId: orchestrationId,
      checkIn: checkIn,
      timeout: timeout,
    );
  }

  @override
  bool get isDisposed => _chainAbstractionClient.isDisposed;

  @override
  void dispose() {
    _chainAbstractionClient.dispose();
  }
}
