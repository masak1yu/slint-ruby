require "minitest/autorun"
require "slint"

class TestColor < Minitest::Test
  def test_new_rgb
    c = Slint::Color.new(255, 128, 0)
    assert_equal 255, c.red
    assert_equal 128, c.green
    assert_equal 0, c.blue
    assert_equal 255, c.alpha
  end

  def test_rgba
    c = Slint::Color.rgba(255, 128, 0, 200)
    assert_equal 200, c.alpha
  end

  def test_brighter
    c = Slint::Color.new(100, 100, 100)
    b = c.brighter(0.5)
    assert b.red > 100
  end

  def test_darker
    c = Slint::Color.new(200, 200, 200)
    d = c.darker(0.5)
    assert d.red < 200
  end

  def test_to_s
    c = Slint::Color.new(255, 0, 128)
    assert_match(/rgba/, c.to_s)
  end
end

class TestBrush < Minitest::Test
  def test_from_color
    c = Slint::Color.new(255, 0, 0)
    b = Slint::Brush.from_color(c)
    assert_equal 255, b.color.red
    assert_equal 0, b.color.green
  end

  def test_brush_brighter
    c = Slint::Color.new(100, 100, 100)
    b = Slint::Brush.from_color(c)
    b2 = b.brighter(0.5)
    assert b2.color.red > 100
  end

  def test_to_s
    c = Slint::Color.new(0, 255, 0)
    b = Slint::Brush.from_color(c)
    assert_match(/Brush/, b.to_s)
  end
end

class TestBrushWithComponent < Minitest::Test
  SLINT_SRC = <<~SLINT
    export component TestComp inherits Window {
      in-out property <brush> bg: red;
    }
  SLINT

  def setup
    compiler = Slint::Compiler.new
    result = compiler.build_from_source(SLINT_SRC, "test.slint")
    refute result.has_errors?
    defn = result.component("TestComp")
    @instance = defn.create
  end

  def test_get_brush_property
    val = @instance.get_property("bg")
    assert_instance_of Slint::Brush, val
  end

  def test_set_brush_property
    c = Slint::Color.new(0, 255, 0)
    b = Slint::Brush.from_color(c)
    @instance.set_property("bg", b)
    val = @instance.get_property("bg")
    assert_instance_of Slint::Brush, val
    assert_equal 0, val.color.red
    assert_equal 255, val.color.green
  end

  def test_set_color_as_brush
    c = Slint::Color.new(0, 0, 255)
    @instance.set_property("bg", c)
    val = @instance.get_property("bg")
    assert_equal 0, val.color.red
    assert_equal 0, val.color.green
    assert_equal 255, val.color.blue
  end
end
