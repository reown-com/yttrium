// ignore_for_file: avoid_print

import 'dart:convert';
import 'dart:io';
import 'dart:isolate';

// flutter pub run yttrium_dart:setup

enum _Environment { ios, android }

void main() async {
  print('Running Yttrium setup script. This could take a while...');
  // Locate the package directory
  final packagePath = await _getPackageRoot();
  if (packagePath == null) {
    print('Error: Could not locate the package directory.');
    exit(1);
  }

  const version = '0.4.5'; // TODO dynamically
  await Future.wait([
    _setupFiles(
      targetDir: '${packagePath.path}/android/src/main/',
      version: version,
      platform: _Environment.android,
    ),
    _setupFiles(
      targetDir: '${packagePath.path}/ios/',
      version: version,
      platform: _Environment.ios,
    ),
  ]);
  //
  print('✅ Yttrium setup success!');
}

Future<void> _setupFiles({
  required String targetDir,
  required String version,
  required _Environment platform,
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

    if (response.statusCode == 200) {
      await response.pipe(File(zipFile).openWrite());
    } else {
      throw Exception(
        'Failed to download file. Status code: ${response.statusCode}',
      );
    }

    // Step 2: Unzip the file using system command
    final args = platform == _Environment.ios
        ? [
            '-o',
            '-j',
            zipFile,
            'universal/libyttrium_lib_universal.dylib',
            '-d',
            targetDir,
          ]
        : ['-o', zipFile, '-d', targetDir];
    final result = await Process.run('unzip', args);
    if (result.exitCode != 0) {
      print('❌ $platform error: ${result.stderr}');
    }
  } catch (e) {
    print('❌ $platform error: $e');
  } finally {
    // Cleanup temporary file
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
        if (package['name'] == 'yttrium_lib') {
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
