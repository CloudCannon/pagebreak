# frozen_string_literal: true

$LOAD_PATH.unshift File.expand_path("lib", __dir__)
require "jekyll-pagebreak/version"

Gem::Specification.new do |spec|
  spec.name          = "jekyll-pagebreak"
  spec.version       = JekyllPagebreak::VERSION
  spec.authors       = ["Liam Bigelow"]
  spec.email         = ["liam@cloudcannon.com"]
  spec.homepage      = "https://github.com/CloudCannon/pagebreak"
  spec.summary       = "A Jekyll plugin to paginate the output of any static site generator"

  spec.files         = `git ls-files lib exe`.split("\n")
  spec.platform      = Gem::Platform::RUBY
  spec.require_paths = ["lib"]
  spec.license       = "MIT"

  spec.add_dependency "jekyll", ">= 3.7", "< 5.0"
  spec.add_development_dependency "rake", ">= 12.3.3"
  spec.add_development_dependency "rubocop-jekyll", "~> 0.5.1"
end
