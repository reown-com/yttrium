Pod::Spec.new do |s|
  s.name         = 'YttriumWrapper'
  s.version      = '0.1.0'
  s.summary      = '4337 implementation'
  s.description  = <<-DESC
    YttriumWrapper is a Swift library that provides an implementation of ERC-4337 for Account Abstraction.
    It leverages a precompiled Rust library to deliver enhanced performance and security for managing Ethereum accounts.
  DESC
  s.homepage     = 'https://reown.com'
  s.license      = { :type => 'MIT', :file => 'LICENSE' }
  s.authors      = { 'Reown, Inc.' => 'contact@reown.com' }

  # Use your Git repository as the source for the podspec
  s.source       = { :git => 'https://github.com/reown-com/yttrium.git', :tag => s.version.to_s }

  s.platform     = :ios, '13.0'

  s.swift_version = '5.9'

  # Include the Swift source files
  s.source_files = 'crates/ffi/YttriumCore/Sources/YttriumCore/**/*.{swift,h}'

  # Remove the exclude_files directive (since the .xcframework isn't in your repo)
  # s.exclude_files = 'crates/ffi/YttriumCore/RustXcframework.xcframework'

  # Since the framework isn't included in the repo, we need to download it
s.prepare_command = <<-SCRIPT
  curl -L -o RustXcframework.xcframework.zip 'https://github.com/reown-com/yttrium/releases/download/0.1.0/RustXcframework.xcframework.zip'
  unzip -o RustXcframework.xcframework.zip -d crates/ffi/YttriumCore/
  rm RustXcframework.xcframework.zip
SCRIPT

  # Now specify the path to the downloaded .xcframework
  s.vendored_frameworks = 'crates/ffi/YttriumCore/RustXcframework.xcframework'
end
