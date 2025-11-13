Pod::Spec.new do |s|
  s.name         = 'YttriumWrapper'
  s.version      = '0.9.111'
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

  # Include the vendored framework with flattened structure
s.prepare_command = <<-SCRIPT
  curl -L -o libyttrium.xcframework.zip 'https://github.com/reown-com/yttrium/releases/download/0.9.111/libyttrium.xcframework.zip'
  unzip -o libyttrium.xcframework.zip -d platforms/swift/
  rm libyttrium.xcframework.zip

  # Restructure the headers if needed
  if [ -d "platforms/swift/target/ios/libyttrium.xcframework/ios-arm64/Headers/yttriumFFI" ]; then
    mv platforms/swift/target/ios/libyttrium.xcframework/ios-arm64/Headers/yttriumFFI/* platforms/swift/target/ios/libyttrium.xcframework/ios-arm64/Headers/
    rm -rf platforms/swift/target/ios/libyttrium.xcframework/ios-arm64/Headers/yttriumFFI
  fi

  if [ -d "platforms/swift/target/ios/libyttrium.xcframework/ios-arm64_x86_64-simulator/Headers/yttriumFFI" ]; then
    mv platforms/swift/target/ios/libyttrium.xcframework/ios-arm64_x86_64-simulator/Headers/yttriumFFI/* platforms/swift/target/ios/libyttrium.xcframework/ios-arm64_x86_64-simulator/Headers/
    rm -rf platforms/swift/target/ios/libyttrium.xcframework/ios-arm64_x86_64-simulator/Headers/yttriumFFI
  fi

  # Copy Swift source files directly to Headers directory for both architectures
  cp -R platforms/swift/Sources/Yttrium/*.swift platforms/swift/target/ios/libyttrium.xcframework/ios-arm64/Headers/
  cp -R platforms/swift/Sources/Yttrium/*.swift platforms/swift/target/ios/libyttrium.xcframework/ios-arm64_x86_64-simulator/Headers/
SCRIPT


  s.vendored_frameworks = 'platforms/swift/target/ios/libyttrium.xcframework'
end
