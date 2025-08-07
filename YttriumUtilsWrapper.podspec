Pod::Spec.new do |spec|
  spec.name         = "YttriumUtilsWrapper"
  spec.version      = "0.0.1"
  spec.summary      = "Yttrium Utils - Multi-blockchain utilities for EIP155, Stacks, and Chain Abstraction"
  spec.description  = <<-DESC
                   Yttrium Utils provides multi-blockchain utilities including EIP155 support, Stacks integration, 
                   and Chain Abstraction capabilities without the full Yttrium Core dependencies.
                   DESC

  spec.homepage     = "https://github.com/reown-com/yttrium"
  spec.license      = { :type => "Apache 2.0", :file => "LICENSE" }
  spec.author       = { "Reown" => "contact@reown.com" }

  spec.ios.deployment_target = "13.0"
  spec.swift_version = "5.9"

  spec.source       = { :git => "https://github.com/reown-com/yttrium.git", :tag => "#{spec.version}" }

  spec.vendored_frameworks = "platforms/swift/target/ios-utils/libyttrium-utils.xcframework"
  spec.source_files = "platforms/swift/Sources/YttriumUtils/**/*.swift"

  # Since this is a utils library with fewer dependencies, we don't need complex configuration
  spec.user_target_xcconfig = {
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'arm64'
  }
  spec.pod_target_xcconfig = {
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'arm64'
  }

  spec.prepare_command = <<-SCRIPT
    curl -L -o libyttrium-utils.xcframework.zip 'https://github.com/reown-com/yttrium/releases/download/#{spec.version}/libyttrium-utils.xcframework.zip'
    unzip -o libyttrium-utils.xcframework.zip -d platforms/swift/
    rm libyttrium-utils.xcframework.zip

    # Restructure the headers if needed for utils
    if [ -d "platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64/Headers/yttriumFFI" ]; then
      mv platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64/Headers/yttriumFFI/* platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64/Headers/
      rm -rf platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64/Headers/yttriumFFI
    fi

    if [ -d "platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64_x86_64-simulator/Headers/yttriumFFI" ]; then
      mv platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64_x86_64-simulator/Headers/yttriumFFI/* platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64_x86_64-simulator/Headers/
      rm -rf platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64_x86_64-simulator/Headers/yttriumFFI
    fi

    # Copy Swift source files directly to Headers directory for both architectures
    cp -R platforms/swift/Sources/YttriumUtils/*.swift platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64/Headers/
    cp -R platforms/swift/Sources/YttriumUtils/*.swift platforms/swift/target/ios-utils/libyttrium-utils.xcframework/ios-arm64_x86_64-simulator/Headers/
  SCRIPT
end 