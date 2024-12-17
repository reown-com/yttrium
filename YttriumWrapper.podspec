Pod::Spec.new do |s|
  s.name         = 'YttriumWrapper'
  s.version      = '0.4.7'
  s.summary      = '4337 implementation'
  s.description  = '4337 implementation and Chain Abstraction'
  s.homepage     = 'https://reown.com'
  s.license      = { :type => 'MIT', :file => 'LICENSE' }
  s.authors      = 'reown inc.'

  s.source       = { :git => 'https://github.com/reown-com/yttrium.git', :tag => s.version.to_s }

  s.platform     = :ios, '13.0'

  s.swift_version = '5.9'

  # Include the Swift source files
  s.source_files = 'platforms/swift/Sources/Yttrium/**/*.{swift,h}'

  # Include the vendored framework
  s.prepare_command = <<-SCRIPT
    curl -L -o libuniffi_yttrium.xcframework.zip 'https://github.com/reown-com/yttrium/releases/download/0.4.7/libuniffi_yttrium.xcframework.zip'
    unzip -o libuniffi_yttrium.xcframework.zip -d platforms/swift/
    rm libuniffi_yttrium.xcframework.zip
  SCRIPT

  s.vendored_frameworks = 'platforms/swift/target/ios/libuniffi_yttrium.xcframework'
end
