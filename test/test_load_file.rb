require "minitest/autorun"
require "slint"

class TestLoadSource < Minitest::Test
  SLINT_SRC = <<~SLINT
    export component App inherits Window {
      in-out property <string> label: "Hello";
      in-out property <int> counter: 0;
      in-out property <bool> active: true;
      callback clicked(string) -> string;
      callback notify();
    }
  SLINT

  def test_load_source_returns_module
    ns = Slint.load_source(SLINT_SRC)
    assert_kind_of Module, ns
  end

  def test_load_source_has_component
    ns = Slint.load_source(SLINT_SRC)
    assert ns.const_defined?(:App)
  end

  def test_component_inherits_from_component
    ns = Slint.load_source(SLINT_SRC)
    assert ns::App < Slint::Component
  end

  def test_create_instance
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    assert_instance_of ns::App, app
  end

  def test_property_getter
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    assert_equal "Hello", app.label
  end

  def test_property_setter
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    app.label = "World"
    assert_equal "World", app.label
  end

  def test_initial_kwargs
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new(label: "Custom", counter: 10)
    assert_equal "Custom", app.label
    assert_equal 10.0, app.counter
  end

  def test_callback_on_handler
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    app.on_clicked { |name| "Clicked: #{name}" }
    result = app.invoke_clicked("button")
    assert_equal "Clicked: button", result
  end

  def test_callback_on_handler_with_proc
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    app.on_clicked(proc { |name| "Proc: #{name}" })
    result = app.invoke_clicked("test")
    assert_equal "Proc: test", result
  end

  def test_void_callback
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    called = false
    app.on_notify { called = true }
    app.invoke_notify
    assert called
  end

  def test_compilation_error
    assert_raises(RuntimeError) do
      Slint.load_source("invalid slint ???")
    end
  end
end

class TestGlobals < Minitest::Test
  SLINT_SRC = <<~SLINT
    export global AppState {
      in-out property <string> current-page: "home";
      callback navigate(string);
    }

    export component App inherits Window {
      in-out property <string> app-name: "App";
    }
  SLINT

  def test_global_accessor
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    state = app.AppState
    assert_respond_to state, :current_page
  end

  def test_global_property_get
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    assert_equal "home", app.AppState.current_page
  end

  def test_global_property_set
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    app.AppState.current_page = "settings"
    assert_equal "settings", app.AppState.current_page
  end

  def test_global_callback
    ns = Slint.load_source(SLINT_SRC)
    app = ns::App.new
    navigated_to = nil
    app.AppState.on_navigate { |page| navigated_to = page }
    app.AppState.invoke_navigate("about")
    assert_equal "about", navigated_to
  end
end
