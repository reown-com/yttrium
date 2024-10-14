Pod::Spec.new do |s|
  s.name         = 'YttriumWrapper'
  s.version      = '0.1.1'
  s.summary      = '4337 implementation'
  s.description  = '4337 implementation '
  s.homepage     = 'https://reown.com'
  s.license      = { :type => 'MIT', :file => 'LICENSE' }
  s.authors      = 'Reown, Inc.'

  s.source       = { :git => 'https://github.com/reown-com/yttrium.git', :tag => s.version.to_s }

  s.platform     = :ios, '13.0'

  s.swift_version = '5.9'

  # Include the Swift source files
  s.source_files = 'crates/ffi/YttriumCore/Sources/YttriumCore/**/*.{swift,h}'

  # Include the vendored framework
  s.prepare_command = <<-SCRIPT
    curl -L -o RustXcframework.xcframework.zip 'https://github.com/reown-com/yttrium/releases/download/0.1.1/RustXcframework.xcframework.zip'
    unzip -o RustXcframework.xcframework.zip -d crates/ffi/YttriumCore/
    rm RustXcframework.xcframework.zip
  SCRIPT

  s.vendored_frameworks = 'crates/ffi/YttriumCore/RustXcframework.xcframework'
end
