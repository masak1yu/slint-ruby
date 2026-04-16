require "minitest/autorun"
require "slint"

class TestCallbacks < Minitest::Test
  SLINT_SRC = <<~SLINT
    export component TestComp inherits Window {
      in-out property <string> result: "";
      callback greet(string) -> string;
      callback notify();
    }
  SLINT

  def setup
    compiler = Slint::Compiler.new
    result = compiler.build_from_source(SLINT_SRC, "test.slint")
    refute result.has_errors?, "Compilation failed: #{result.diagnostics.map(&:to_s).join(', ')}"
    defn = result.component("TestComp")
    @instance = defn.create
  end

  def test_set_and_invoke_callback
    @instance.set_callback("greet", proc { |name| "Hello, #{name}!" })
    result = @instance.invoke("greet", ["World"])
    assert_equal "Hello, World!", result
  end

  def test_callback_with_no_return
    called = false
    @instance.set_callback("notify", proc { called = true })
    @instance.invoke("notify", [])
    assert called
  end

  def test_set_callback_with_lambda
    @instance.set_callback("greet", ->(name) { "Hi #{name}" })
    result = @instance.invoke("greet", ["Ruby"])
    assert_equal "Hi Ruby", result
  end

  def test_set_nonexistent_callback
    assert_raises(RuntimeError) do
      @instance.set_callback("nonexistent", proc { })
    end
  end

  def test_invoke_nonexistent
    assert_raises(RuntimeError) do
      @instance.invoke("nonexistent", [])
    end
  end
end
