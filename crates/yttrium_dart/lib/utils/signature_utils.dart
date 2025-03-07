import 'package:eth_sig_util/util/bigint.dart';
import 'package:eth_sig_util/util/signature.dart';
import 'package:eth_sig_util/util/utils.dart';
import 'package:yttrium_dart/generated/chain_abstraction/dart_compat_models.dart';

extension SignatureExtension on String {
  PrimitiveSignatureCompat toPrimitiveSignature() {
    final ecdsaSignature = SignatureUtil.fromRpcSig(this);

    // Extract r, s, and v
    final rHex = bytesToHex(encodeBigInt(ecdsaSignature.r));
    final sHex = bytesToHex(encodeBigInt(ecdsaSignature.s));

    // Convert v to yParity (Ethereum EIP-1559 format)
    final bool yParity = (ecdsaSignature.v == 28);

    return PrimitiveSignatureCompat(
      r: rHex,
      s: sHex,
      yParity: yParity,
    );
  }
}
