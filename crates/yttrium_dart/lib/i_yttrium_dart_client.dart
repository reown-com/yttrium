import 'package:yttrium_dart/generated/chain_abstraction/dart_compat.dart';
import 'package:yttrium_dart/generated/chain_abstraction/dart_compat_models.dart';

abstract class IYttriumClient extends ChainAbstractionClient {
  Future<void> init({
    required String projectId,
    required PulseMetadataCompat pulseMetadata,
  });
}
