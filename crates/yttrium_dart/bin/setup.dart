// ignore_for_file: avoid_print

import 'dart:convert';
import 'dart:io';
import 'dart:isolate';

// flutter pub run yttrium_dart:generate

void main() async {
  print('Running Yttrium setup script...');
  // Locate the package directory
  final packagePath = await _getPackageRoot();
  if (packagePath == null) {
    print('Error: Could not locate the package directory.');
    exit(1);
  }

  // https://github.com/reown-com/yttrium/releases/download/0.0.1/dart-artifacts.zip
  const rootPackage = 'https://github.com/reown-com/yttrium';
  const version = '0.0.2'; // TODO dynamically
  final url = '$rootPackage/releases/download/$version/dart-artifacts.zip';
  //
  final androidTargetDir = '${packagePath.path}android/src/main/';
  final zipFilePath = '${Directory.systemTemp.path}/dart-artifacts.zip';

  try {
    // Step 1: Download the ZIP file
    print('Downloading ZIP file...');
    final request = await HttpClient().getUrl(Uri.parse(url));
    final response = await request.close();

    if (response.statusCode == 200) {
      await response.pipe(File(zipFilePath).openWrite());
      print('Downloaded ZIP file to $zipFilePath');
    } else {
      throw Exception(
        'Failed to download file. Status code: ${response.statusCode}',
      );
    }

    // Step 2: Unzip the file using system command
    print('Unzipping the file...');
    final result = await Process.run(
      'unzip',
      ['-o', zipFilePath, '-d', androidTargetDir],
    );
    // jniLibs/arm64-v8a/libyttrium_dart.so

    if (result.exitCode == 0) {
      print('Unzipped contents to $androidTargetDir');
    } else {
      print('Failed to unzip file.');
      print('Error: ${result.stderr}');
    }
  } catch (e) {
    print('An error occurred: $e');
  } finally {
    // Cleanup temporary file
    File(zipFilePath).deleteSync();
    print('Cleaned up temporary files.');
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
