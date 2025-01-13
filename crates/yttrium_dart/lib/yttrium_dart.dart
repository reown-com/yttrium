import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:yttrium_dart/generated/frb_generated.dart' as frb;
import 'package:yttrium_dart/generated/lib.dart';

class YttriumDart implements ChainAbstractionClient {
  // Singleton instance
  static final YttriumDart _instance = YttriumDart._internal();

  // Public accessor for the singleton instance
  static YttriumDart get instance => _instance;

  // Private constructor
  YttriumDart._internal();

  ChainAbstractionClient? _chainAbstractionClient;

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
      );
    } catch (e) {
      rethrow;
    }
  }

  // TODO shouldn't be needed
  @override
  String get projectId {
    if (_chainAbstractionClient == null) {
      throw 'ChainAbstractionClient is not initialized';
    }

    return _chainAbstractionClient!.projectId;
  }

  // TODO shouldn't be needed
  @override
  bool get isDisposed => _chainAbstractionClient?.isDisposed ?? true;

  // TODO shouldn't be needed
  @override
  set projectId(String projectId) {
    if (_chainAbstractionClient == null) {
      throw 'ChainAbstractionClient is not initialized';
    }
    _chainAbstractionClient!.projectId = projectId;
  }

  @override
  Future<Eip1559Estimation> estimateFees({required String chainId}) async {
    if (_chainAbstractionClient == null) {
      throw 'ChainAbstractionClient is not initialized';
    }
    return await _chainAbstractionClient!.estimateFees(
      chainId: chainId,
    );
  }

  // @override
  // Future<PrepareResponse> prepare({
  //   required InitialTransaction initialTransaction,
  // }) async {
  //   if (_chainAbstractionClient == null) {
  //     throw 'ChainAbstractionClient is not initialized';
  //   }
  //   return await _chainAbstractionClient!.prepare(
  //     initialTransaction: initialTransaction,
  //   );
  // }

  @override
  Future<PrepareResponse> prepare({
    required String chainId,
    required Address from,
    required Call call,
  }) async {
    if (_chainAbstractionClient == null) {
      throw 'ChainAbstractionClient is not initialized';
    }
    return await _chainAbstractionClient!.prepare(
      chainId: chainId,
      from: from,
      call: call,
    );
  }

  @override
  Future<String> erc20TokenBalance({
    required String chainId,
    required Address token,
    required Address owner,
  }) async {
    if (_chainAbstractionClient == null) {
      throw 'ChainAbstractionClient is not initialized';
    }
    return await _chainAbstractionClient!.erc20TokenBalance(
      chainId: chainId,
      token: token,
      owner: owner,
    );
  }

  @override
  Future<UiFields> getUiFields({
    required PrepareResponseAvailable routeResponse,
    required Currency currency,
  }) async {
    if (_chainAbstractionClient == null) {
      throw 'ChainAbstractionClient is not initialized';
    }
    return await _chainAbstractionClient!.getUiFields(
      routeResponse: routeResponse,
      currency: currency,
    );
  }

  @override
  Future<StatusResponse> status({required String orchestrationId}) async {
    if (_chainAbstractionClient == null) {
      throw 'ChainAbstractionClient is not initialized';
    }
    return await _chainAbstractionClient!.status(
      orchestrationId: orchestrationId,
    );
  }

  @override
  Future<StatusResponseCompleted> waitForSuccessWithTimeout({
    required String orchestrationId,
    required BigInt checkIn,
    required BigInt timeout,
  }) async {
    if (_chainAbstractionClient == null) {
      throw 'ChainAbstractionClient is not initialized';
    }
    return await _chainAbstractionClient!.waitForSuccessWithTimeout(
      orchestrationId: orchestrationId,
      checkIn: checkIn,
      timeout: timeout,
    );
  }

  // TODO shouldn't be needed
  @override
  void dispose() {
    _chainAbstractionClient?.dispose();
  }
}
