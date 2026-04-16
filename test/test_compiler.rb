require "minitest/autorun"
require "slint"

class TestCompiler < Minitest::Test
  def test_compiler_new
    compiler = Slint::Compiler.new
    assert_instance_of Slint::Compiler, compiler
  end

  def test_build_from_source
    compiler = Slint::Compiler.new
    result = compiler.build_from_source('export component Foo inherits Window {}', "test.slint")
    refute result.has_errors?
    assert_includes result.component_names, "Foo"
  end

  def test_component_definition
    compiler = Slint::Compiler.new
    result = compiler.build_from_source('export component Foo inherits Window {}', "test.slint")
    defn = result.component("Foo")
    assert_equal "Foo", defn.name
  end

  def test_component_not_found
    compiler = Slint::Compiler.new
    result = compiler.build_from_source('export component Foo inherits Window {}', "test.slint")
    assert_nil result.component("Bar")
  end

  def test_diagnostics_on_error
    compiler = Slint::Compiler.new
    result = compiler.build_from_source('invalid slint code ???', "test.slint")
    assert result.has_errors?
    refute_empty result.diagnostics
  end

  def test_component_definition_properties
    compiler = Slint::Compiler.new
    result = compiler.build_from_source(<<~SLINT, "test.slint")
      export component Foo inherits Window {
        in property <string> label: "hello";
        in property <int> count: 0;
      }
    SLINT
    defn = result.component("Foo")
    prop_names = defn.properties.map(&:first)
    assert_includes prop_names, "label"
    assert_includes prop_names, "count"
  end

  def test_component_create
    compiler = Slint::Compiler.new
    result = compiler.build_from_source('export component Foo inherits Window {}', "test.slint")
    defn = result.component("Foo")
    instance = defn.create
    assert_instance_of Slint::ComponentInstance, instance
  end

  def test_include_paths
    compiler = Slint::Compiler.new
    compiler.include_paths = ["/tmp"]
    assert_equal ["/tmp"], compiler.include_paths
  end

  def test_style
    compiler = Slint::Compiler.new
    compiler.style = "fluent"
    assert_equal "fluent", compiler.style
  end
end
