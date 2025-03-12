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
  s.author           = { 'Reown' => 'mobile@reown.com' }
  s.source           = { :path => '.' }
  s.source_files = 'Classes/**/*'
  s.dependency 'Flutter'
  s.platform            = :ios, '13.0'

  # Explicitly reference `yttrium.xcframework` to prevent renaming
  s.vendored_frameworks = 'yttrium_dart.xcframework'
  # # Prevent CocoaPods from renaming the framework
  s.module_name = 'yttrium_dart'
  # Preserve the xcframework structure
  s.preserve_paths = 'yttrium_dart.xcframework/**/*'

  s.swift_version = '5.0'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    # Flutter.framework does not contain a i386 slice.
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386',
    'ENABLE_BITCODE' => 'NO',
    'STRIP_INSTALLED_PRODUCT' => 'NO',
    'STRIP_STYLE' => 'non-global',
    'DEAD_CODE_STRIPPING' => 'NO',
    'STRIP_SWIFT_SYMBOLS' => 'NO',
  }

  # s.xcconfig = { 'OTHER_LDFLAGS' => '-framework yttrium' }

  # If your plugin requires a privacy manifest, for example if it uses any
  # required reason APIs, update the PrivacyInfo.xcprivacy file to describe your
  # plugin's privacy impact, and then uncomment this line. For more information,
  # see https://developer.apple.com/documentation/bundleresources/privacy_manifest_files
  # s.resource_bundles = {'yttrium_dart_privacy' => ['Resources/PrivacyInfo.xcprivacy']}
end
