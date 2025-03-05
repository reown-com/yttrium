// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.8.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

enum BridgingError {
  noRoutesAvailable,
  insufficientFunds,
  insufficientGasFunds,
  ;
}

/// Bridging check error response that should be returned as a normal HTTP 200
/// response
class PrepareResponseError {
  final BridgingError error;
  final String reason;

  const PrepareResponseError({
    required this.error,
    required this.reason,
  });

  @override
  int get hashCode => error.hashCode ^ reason.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PrepareResponseError &&
          runtimeType == other.runtimeType &&
          error == other.error &&
          reason == other.reason;
}
