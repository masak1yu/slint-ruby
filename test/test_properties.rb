require "minitest/autorun"
require "slint"

class TestProperties < Minitest::Test
  SLINT_SRC = <<~SLINT
    export component TestComp inherits Window {
      in-out property <string> label: "hello";
      in-out property <int> count: 42;
      in-out property <float> ratio: 3.14;
      in-out property <bool> is-active: true;
    }
  SLINT

  def setup
    compiler = Slint::Compiler.new
    result = compiler.build_from_source(SLINT_SRC, "test.slint")
    refute result.has_errors?, "Compilation failed: #{result.diagnostics.map(&:to_s).join(', ')}"
    defn = result.component("TestComp")
    @instance = defn.create
  end

  def test_get_string_property
    assert_equal "hello", @instance.get_property("label")
  end

  def test_set_string_property
    @instance.set_property("label", "world")
    assert_equal "world", @instance.get_property("label")
  end

  def test_get_int_property
    assert_equal 42.0, @instance.get_property("count")
  end

  def test_set_int_property
    @instance.set_property("count", 100)
    assert_equal 100.0, @instance.get_property("count")
  end

  def test_get_float_property
    assert_in_delta 3.14, @instance.get_property("ratio"), 0.001
  end

  def test_set_float_property
    @instance.set_property("ratio", 2.718)
    assert_in_delta 2.718, @instance.get_property("ratio"), 0.001
  end

  def test_get_bool_property
    assert_equal true, @instance.get_property("is-active")
  end

  def test_set_bool_property
    @instance.set_property("is-active", false)
    assert_equal false, @instance.get_property("is-active")
  end

  def test_get_nonexistent_property
    assert_raises(RuntimeError) { @instance.get_property("nonexistent") }
  end
end
