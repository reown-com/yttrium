#
# To learn more about a Podspec see http://guides.cocoapods.org/syntax/podspec.html.
# Run `pod lib lint yttrium_dart.podspec` to validate before publishing.
#
Pod::Spec.new do |s|
  s.name             = 'yttrium_dart'
  s.version          = '0.0.1'
  s.summary          = 'Reown - YttriumDart'
  s.description      = <<-DESC
Reown is the onchain UX platform that provides toolkits built on top of the WalletConnect Network
                       DESC
  s.homepage         = 'https://reown.com'
  s.license          = { :file => '../LICENSE' }
  s.author           = { 'Your Company' => 'mobile@reown.com' }
  s.source           = { :path => '.' }
  s.source_files = 'Classes/**/*'
  s.vendored_libraries = 'libyttrium_dart_universal.dylib'
  s.dependency 'Flutter'
  s.platform = :ios, '12.0'

  # Flutter.framework does not contain a i386 slice.
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386',
    'LD_RUNPATH_SEARCH_PATHS' => '$(inherited) @executable_path/Frameworks',
  }
  s.swift_version = '5.0'

  # If your plugin requires a privacy manifest, for example if it uses any
  # required reason APIs, update the PrivacyInfo.xcprivacy file to describe your
  # plugin's privacy impact, and then uncomment this line. For more information,
  # see https://developer.apple.com/documentation/bundleresources/privacy_manifest_files
  # s.resource_bundles = {'yttrium_dart_privacy' => ['Resources/PrivacyInfo.xcprivacy']}
end
