# coding: utf-8
lib = File.expand_path('../lib', __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require 'superconductor/version'

Gem::Specification.new do |spec|
  spec.name          = "superconductor"
  spec.version       = Superconductor::VERSION
  spec.authors       = ["Jaden Carver"]
  spec.email         = ["jaden.carver@gmail.com"]

  spec.summary       = %q{Development Integrated Project Management}
  spec.description   = %q{Project Management tools that integrate into your developer workflow}
  spec.homepage      = "https://www.github.com/jadencarver/superconductor"
  spec.license       = "MIT"

  # Prevent pushing this gem to RubyGems.org. To allow pushes either set the 'allowed_push_host'
  # to allow pushing to a single host or delete this section to allow pushing to any host.
  if spec.respond_to?(:metadata)
    spec.metadata['allowed_push_host'] = "https://rubygems.org"
  else
    raise "RubyGems 2.0 or newer is required to protect against public gem pushes."
  end

  spec.files         = `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  spec.add_development_dependency "bundler", "~> 2.1"
  spec.add_development_dependency "rake", ">= 12.3.3"
  spec.add_development_dependency "rake", "~> 10.0"
  spec.add_development_dependency "yard", "~> 0.9.2"
  spec.add_development_dependency "rspec", "~> 3.0"
  spec.add_development_dependency "selenium-webdriver", "~> 3.4.0"
  spec.add_development_dependency "poltergeist", "~> 1.15.0"
  spec.add_development_dependency "capybara-screenshot", "~> 0.3.3"
  spec.add_development_dependency "ansi", "~> 1.5.0"
  spec.add_development_dependency "mini_magick", "~> 4.10.1"
  spec.add_development_dependency "faker", "~> 1.7.3"
  spec.extensions = Dir['extconf.rb']
end
