// ignore_for_file: avoid_print

import 'dart:convert';
import 'dart:io';
import 'dart:isolate';

// dart run yttrium_dart:setup --sim --version=0.0.1

enum _Environment { ios, android }

void main(List<String> args) async {
  if (args.isEmpty || args.length > 2) {
    print('Error: The only valid command formats are:\n');
    print('  dart run yttrium_dart:setup --version=X.Y.Z\n'
        '  dart run yttrium_dart:setup --sim --version=X.Y.Z');
    exit(1);
  }

  // Find the --version argument
  final versionArg =
      args.firstWhere((arg) => arg.startsWith('--version='), orElse: () => '');

  if (versionArg.isEmpty) {
    print("Error: Missing required '--version=X.Y.Z' argument.");
    exit(1);
  }

  // Extract version number
  final versionValue = versionArg.split('=').last;

  if (versionValue.isEmpty) {
    print(
        "Error: '--version' argument must have a value (e.g., '--version=0.4.5').");
    exit(1);
  }

  // Validate allowed arguments (only --sim and --version=X.Y.Z)
  final validArgs = {'--sim', versionArg};
  if (args.toSet().difference(validArgs).isNotEmpty) {
    print('Error: Invalid arguments detected.');
    exit(1);
  }
  print('Running setup with version: $versionValue');

  final bool isSimulator = args.contains('--sim');
  if (isSimulator) {
    print('Running for simulator.');
  }

  print('Running Yttrium setup. This could take a while...');
  // Locate the package directory
  final packagePath = await _getPackageRoot();
  print('packagePath: ${packagePath?.path}');
  if (packagePath == null) {
    print('Error: Could not locate the package directory.');
    exit(1);
  }

  await Future.wait([
    _setupFiles(
      targetDir: '${packagePath.path}/android/src/main/',
      version: versionValue,
      platform: _Environment.android,
      isSimulator: isSimulator,
    ),
    _setupFiles(
      targetDir: '${packagePath.path}/ios/',
      version: versionValue,
      platform: _Environment.ios,
      isSimulator: isSimulator,
    ),
  ]);
  //
  print('✅ Yttrium setup success!');
}

Future<void> _setupFiles({
  required String targetDir,
  required String version,
  required _Environment platform,
  required bool isSimulator,
}) async {
  final artifactFile = '${platform.name}-artifacts.zip';
  final releases =
      'https://github.com/reown-com/yttrium/releases/download/$version/';
  final downloadUrl = '$releases$artifactFile';
  final zipFile = '${Directory.systemTemp.path}/$artifactFile';

  try {
    // Step 1: Download the ZIP file
    final request = await HttpClient().getUrl(Uri.parse(downloadUrl));
    final response = await request.close();

    if (response.statusCode != 200) {
      throw Exception(
        'Failed to download file. Status code: ${response.statusCode}',
      );
    }

    await response.pipe(File(zipFile).openWrite());

    // Step 2: Unzip the required file
    late final List<String> args;
    late final String extractedFile;
    late final String finalFile;

    if (platform == _Environment.ios) {
      // Determine which file to extract based on isSimulator flag
      final extractedFileName = isSimulator
          ? 'universal/libyttrium_universal_sim.dylib'
          : 'universal/libyttrium_universal.dylib';

      extractedFile = '$targetDir/${extractedFileName.split('/').last}';
      finalFile = '$targetDir/libyttrium_universal.dylib';

      args = ['-o', '-j', zipFile, extractedFileName, '-d', targetDir];
    } else {
      // Extract the full archive
      args = ['-o', zipFile, '-d', targetDir];
    }

    final result = await Process.run('unzip', args);

    if (result.exitCode != 0) {
      print('❌ $platform error: ${result.stderr}');
      return;
    }

    // Step 3: Rename the extracted file if needed (only for iOS)
    if (platform == _Environment.ios) {
      final file = File(extractedFile);
      if (await file.exists()) {
        await file.rename(finalFile);
        print('✅ Renamed $extractedFile → $finalFile');
      } else {
        print('❌ File not found for renaming: $extractedFile');
      }
    }
  } catch (e) {
    print('❌ $platform error: $e');
  } finally {
    // Cleanup temporary ZIP file
    File(zipFile).deleteSync();
    print('Cleaning up unneeded files...');
  }
}

Future<Directory?> _getPackageRoot() async {
  try {
    // Get the package configuration file
    // Locate the package configuration file
    final packageConfigUri = await Isolate.packageConfig;
    if (packageConfigUri == null) {
      print('Error: Could not locate the package configuration file.');
      exit(1);
    }
    final packageConfigFile = File.fromUri(packageConfigUri);

    // Read and parse the package_config.json file
    final jsonContent = jsonDecode(await packageConfigFile.readAsString());
    final packages = jsonContent['packages'] as List<dynamic>?;

    if ((packages ?? []).isNotEmpty) {
      for (final package in packages!) {
        if (package['name'] == 'yttrium_dart') {
          final rootUri = package['rootUri'] as String?;
          if (rootUri != null) {
            // Resolve absolute path for relative URIs
            final resolvedUri = Uri.parse(rootUri);
            if (resolvedUri.isAbsolute) {
              return Directory.fromUri(resolvedUri);
            } else {
              // Resolve relative path based on the package_config.json location
              final packageConfigDir = packageConfigFile.parent;
              return Directory.fromUri(
                packageConfigDir.uri.resolveUri(resolvedUri),
              );
            }
          }
        }
      }
    }
  } catch (e) {
    print('Error locating package root: $e');
  }
  return null;
}
