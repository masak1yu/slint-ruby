require "rake/extensiontask"
require "rake/testtask"

Rake::ExtensionTask.new("slint_ruby") do |ext|
  ext.lib_dir = "lib/slint"
  ext.source_pattern = "*.{rs,toml}"
end

Rake::TestTask.new do |t|
  t.libs << "lib"
  t.libs << "test"
  t.test_files = FileList["test/test_*.rb"]
end

task default: [:compile, :test]
