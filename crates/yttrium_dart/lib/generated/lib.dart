// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.7.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import 'frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart' hide protected;
part 'lib.freezed.dart';

// These types are ignored because they are not used by any `pub` functions: `PreparedSignature`, `transferReturn`
// These function are ignored because they are on traits that is not defined in current crate (put an empty `#[frb]` on it to unignore): `clone`, `clone`, `clone`, `clone`, `clone`, `fmt`, `fmt`, `fmt`, `fmt`, `fmt`, `from`, `from`, `try_from`

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<:: Address>>
abstract class Address implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<ChainAbstractionClient>>
abstract class ChainAbstractionClient implements RustOpaqueInterface {
  String get projectId;

  set projectId(String projectId);

  Future<String> erc20TokenBalance(
      {required String chainId, required String token, required String owner});

  Future<Eip1559Estimation> estimateFees({required String chainId});

  Future<UiFields> getUiFields(
      {required PrepareResponseAvailable routeResponse,
      required Currency currency});

  // HINT: Make it `#[frb(sync)]` to let it become the default constructor of Dart class.
  static Future<ChainAbstractionClient> newInstance(
          {required String projectId}) =>
      YttriumDart.instance.api
          .crateChainAbstractionClientNew(projectId: projectId);

  Future<PrepareResponse> prepare(
      {required String chainId, required String from, required FFICall call});

  Future<FFICall> prepareErc20TransferCall(
      {required String erc20Address,
      required String to,
      required BigInt amount});

  Future<StatusResponse> status({required String orchestrationId});

  Future<StatusResponseCompleted> waitForSuccessWithTimeout(
      {required String orchestrationId,
      required BigInt checkIn,
      required BigInt timeout});
}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<Currency>>
abstract class Currency implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<PrepareResponse>>
abstract class PrepareResponse implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<PrepareResponseAvailable>>
abstract class PrepareResponseAvailable implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<StatusResponse>>
abstract class StatusResponse implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<StatusResponseCompleted>>
abstract class StatusResponseCompleted implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<:: U256>>
abstract class U256 implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<UiFields>>
abstract class UiFields implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<transferCall>>
abstract class TransferCall implements RustOpaqueInterface {
  U256 get amount;

  Address get recipient;

  set amount(U256 amount);

  set recipient(Address recipient);
}

class Eip1559Estimation {
  /// The base fee per gas as a String.
  final String maxFeePerGas;

  /// The max priority fee per gas as a String.
  final String maxPriorityFeePerGas;

  const Eip1559Estimation({
    required this.maxFeePerGas,
    required this.maxPriorityFeePerGas,
  });

  @override
  int get hashCode => maxFeePerGas.hashCode ^ maxPriorityFeePerGas.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Eip1559Estimation &&
          runtimeType == other.runtimeType &&
          maxFeePerGas == other.maxFeePerGas &&
          maxPriorityFeePerGas == other.maxPriorityFeePerGas;
}

class FFICall {
  final String to;
  final BigInt value;
  final Uint8List input;

  const FFICall({
    required this.to,
    required this.value,
    required this.input,
  });

  @override
  int get hashCode => to.hashCode ^ value.hashCode ^ input.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FFICall &&
          runtimeType == other.runtimeType &&
          to == other.to &&
          value == other.value &&
          input == other.input;
}

@freezed
sealed class FFIError with _$FFIError implements FrbException {
  const FFIError._();

  const factory FFIError.general(
    String field0,
  ) = FFIError_General;
}
