require "minitest/autorun"
require "slint"

class TestListModel < Minitest::Test
  def test_create_empty
    model = Slint::ListModel.new([])
    assert_equal 0, model.row_count
  end

  def test_create_with_data
    model = Slint::ListModel.new(["a", "b", "c"])
    assert_equal 3, model.row_count
    assert_equal "a", model.row_data(0)
    assert_equal "b", model.row_data(1)
    assert_equal "c", model.row_data(2)
  end

  def test_push
    model = Slint::ListModel.new([])
    model.push("hello")
    assert_equal 1, model.row_count
    assert_equal "hello", model.row_data(0)
  end

  def test_shovel_operator
    model = Slint::ListModel.new([])
    model << "world"
    assert_equal 1, model.size
    assert_equal "world", model.row_data(0)
  end

  def test_set_row_data
    model = Slint::ListModel.new(["old"])
    model.set_row_data(0, "new")
    assert_equal "new", model.row_data(0)
  end

  def test_remove
    model = Slint::ListModel.new(["a", "b", "c"])
    model.remove(1)
    assert_equal 2, model.row_count
    assert_equal "a", model.row_data(0)
    assert_equal "c", model.row_data(1)
  end

  def test_insert
    model = Slint::ListModel.new(["a", "c"])
    model.insert(1, "b")
    assert_equal 3, model.row_count
    assert_equal "b", model.row_data(1)
  end

  def test_clear
    model = Slint::ListModel.new(["a", "b", "c"])
    model.clear
    assert_equal 0, model.row_count
  end

  def test_to_a
    model = Slint::ListModel.new(["x", "y"])
    assert_equal ["x", "y"], model.to_a
  end

  def test_length_and_size
    model = Slint::ListModel.new([1, 2, 3])
    assert_equal 3, model.length
    assert_equal 3, model.size
  end

  def test_out_of_range
    model = Slint::ListModel.new(["only"])
    assert_raises(RangeError) { model.row_data(5) }
    assert_raises(RangeError) { model.remove(5) }
  end

  def test_mixed_types
    model = Slint::ListModel.new([1, "two", 3.0, true])
    assert_equal 4, model.row_count
    assert_in_delta 1.0, model.row_data(0)
    assert_equal "two", model.row_data(1)
    assert_in_delta 3.0, model.row_data(2)
    assert_equal true, model.row_data(3)
  end
end

class TestModelWithComponent < Minitest::Test
  SLINT_SRC = <<~SLINT
    export component TestComp inherits Window {
      in-out property <[string]> items;
    }
  SLINT

  def setup
    compiler = Slint::Compiler.new
    result = compiler.build_from_source(SLINT_SRC, "test.slint")
    refute result.has_errors?
    defn = result.component("TestComp")
    @instance = defn.create
  end

  def test_set_model_property
    model = Slint::ListModel.new(["apple", "banana"])
    @instance.set_property("items", model)
    # Getting a model property back isn't supported yet in value conversion,
    # but setting should not error
  end
end
