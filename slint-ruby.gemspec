Gem::Specification.new do |spec|
  spec.name = "slint-ruby"
  spec.version = "0.1.0"
  spec.authors = ["slint-ruby contributors"]
  spec.summary = "Unofficial Ruby bindings for Slint UI toolkit"
  spec.description = "Ruby bindings for Slint, a declarative GUI toolkit for Rust, C++, Python, and now Ruby."
  spec.homepage = "https://github.com/user/slint-ruby"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.2.0"

  spec.files = Dir["lib/**/*.rb", "ext/**/*.{rb,rs,toml}", "Cargo.lock"]
  spec.extensions = ["ext/slint_ruby/extconf.rb"]
  spec.require_paths = ["lib"]

  spec.add_dependency "rb_sys"
end
