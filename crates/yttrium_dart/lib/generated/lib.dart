// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.7.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import 'frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart' hide protected;
part 'lib.freezed.dart';

// These function are ignored because they are on traits that is not defined in current crate (put an empty `#[frb]` on it to unignore): `clone`, `clone`, `clone`, `clone`, `fmt`, `fmt`, `fmt`, `fmt`, `fmt`, `fmt`
// These functions are ignored (category: IgnoreBecauseNotAllowedOwner): `from`

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<AccountClient>>
abstract class AccountClient implements RustOpaqueInterface {
  BigInt get chainId;

  String get ownerAddress;

  set chainId(BigInt chainId);

  set ownerAddress(String ownerAddress);

  Future<String> doSendTransactions(
      {required List<OwnerSignature> signatures,
      required String doSendTransactionParams});

  Future<String> getAddress();

  Future<BigInt> getChainId();

  // HINT: Make it `#[frb(sync)]` to let it become the default constructor of Dart class.
  static Future<AccountClient> newInstance(
          {required AccountClientConfig config}) =>
      YttriumDart.instance.api.crateAccountClientNew(config: config);

  Future<PreparedSendTransaction> prepareSendTransactions(
      {required List<Transaction> transactions});

  Future<String> waitForUserOperationReceipt(
      {required String userOperationHash});
}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<AccountClientConfig>>
abstract class AccountClientConfig implements RustOpaqueInterface {
  BigInt get chainId;

  Config get config;

  String get ownerAddress;

  set chainId(BigInt chainId);

  set config(Config config);

  set ownerAddress(String ownerAddress);
}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<ChainAbstractionClient>>
abstract class ChainAbstractionClient implements RustOpaqueInterface {
  String get projectId;

  set projectId(String projectId);

  Future<Eip1559Estimation> estimateFees({required String chainId});

  // HINT: Make it `#[frb(sync)]` to let it become the default constructor of Dart class.
  static Future<ChainAbstractionClient> newInstance(
          {required String projectId}) =>
      YttriumDart.instance.api
          .crateChainAbstractionClientNew(projectId: projectId);

  Future<PrepareResponse> route(
      {required InitialTransaction initialTransaction});

  Future<StatusResponse> status({required String orchestrationId});

  Future<StatusResponseCompleted> waitForSuccessWithTimeout(
      {required String orchestrationId,
      required BigInt checkIn,
      required BigInt timeout});
}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<Config>>
abstract class Config implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<InitialTransaction>>
abstract class InitialTransaction implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<PrepareResponse>>
abstract class PrepareResponse implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<StatusResponse>>
abstract class StatusResponse implements RustOpaqueInterface {}

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<StatusResponseCompleted>>
abstract class StatusResponseCompleted implements RustOpaqueInterface {}

class Eip1559Estimation {
  final String maxFeePerGas;
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

@freezed
sealed class Error with _$Error implements FrbException {
  const Error._();

  const factory Error.general(
    String field0,
  ) = Error_General;
}

class OwnerSignature {
  final String owner;
  final String signature;

  const OwnerSignature({
    required this.owner,
    required this.signature,
  });

  @override
  int get hashCode => owner.hashCode ^ signature.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is OwnerSignature &&
          runtimeType == other.runtimeType &&
          owner == other.owner &&
          signature == other.signature;
}

class PreparedSendTransaction {
  final String hash;
  final String doSendTransactionParams;

  const PreparedSendTransaction({
    required this.hash,
    required this.doSendTransactionParams,
  });

  @override
  int get hashCode => hash.hashCode ^ doSendTransactionParams.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PreparedSendTransaction &&
          runtimeType == other.runtimeType &&
          hash == other.hash &&
          doSendTransactionParams == other.doSendTransactionParams;
}

class Transaction {
  final String to;
  final String value;
  final String data;

  const Transaction({
    required this.to,
    required this.value,
    required this.data,
  });

  @override
  int get hashCode => to.hashCode ^ value.hashCode ^ data.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Transaction &&
          runtimeType == other.runtimeType &&
          to == other.to &&
          value == other.value &&
          data == other.data;
}
