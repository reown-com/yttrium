Pod::Spec.new do |spec|
  spec.name         = "YttriumUtilsWrapper"
  spec.version      = "0.9.78"
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

  # Binary pod via :http to avoid running heavy prepare_command on trunk
  # Binary asset hosted on GitHub Releases; CI updates the version
  spec.source       = { :http => "https://github.com/reown-com/yttrium/releases/download/0.9.78/libyttrium-utils-pod.zip" }

  # The zip contains libyttrium-utils.xcframework at root and Sources/YttriumUtils/*.swift
  spec.vendored_frameworks = "libyttrium-utils.xcframework"
  spec.source_files = "Sources/YttriumUtils/**/*.swift"

  # No prepare_command needed for binary pod
end 