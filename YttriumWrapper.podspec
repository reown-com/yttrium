Pod::Spec.new do |s|
  s.name         = 'YttriumWrapper'
  s.version      = '0.8.28'
  s.summary      = '4337 implementation'
  s.description  = '4337 implementation and Chain Abstraction'
  s.homepage     = 'https://reown.com'
  s.license      = { :type => 'MIT', :file => 'LICENSE' }
  s.authors      = 'reown inc.'

  s.source       = { :git => 'https://github.com/reown-com/yttrium.git', :tag => s.version.to_s }
  s.platform     = :ios, '13.0'
  s.swift_version = '5.9'


  s.prepare_command = <<-SCRIPT
    curl -L -o libuniffi_yttrium.xcframework.zip 'https://github.com/reown-com/yttrium/releases/download/0.8.28/libuniffi_yttrium.xcframework.zip'
    unzip -o libuniffi_yttrium.xcframework.zip -d platforms/swift/
    rm libuniffi_yttrium.xcframework.zip
  SCRIPT

  s.vendored_frameworks = 'platforms/swift/target/ios/libuniffi_yttrium.xcframework'

  # Suppress Swift 6 warnings during validation
  s.pod_target_xcconfig = {
    'OTHER_SWIFT_FLAGS' => '-suppress-warnings',
    'SWIFT_SUPPRESS_WARNINGS' => 'YES',
      'EXCLUDED_ARCHS' => '',
  'MODULEMAP_FILE' => '$(PODS_ROOT)/YttriumWrapper/platforms/swift/target/ios/libuniffi_yttrium.xcframework/Headers/module.modulemap'
  }
end
