# slint-ruby

Unofficial Ruby bindings for [Slint](https://slint.dev/), a declarative GUI toolkit.

Built with [Magnus](https://github.com/matsadler/magnus) (Rust FFI for Ruby) wrapping [`slint-interpreter`](https://docs.rs/slint-interpreter/).

> **Status**: Experimental. API may change.

## Requirements

- Ruby >= 3.2
- Rust toolchain (for compiling the native extension)
- System dependencies for Slint's renderer:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libxkbcommon-dev`, `libwayland-dev`

## Installation

Add to your `Gemfile`:

```ruby
gem "slint-ruby", git: "https://github.com/masak1yu/slint-ruby"
```

Then:

```sh
bundle install
```

## Quick Start

```ruby
require "slint"

# Define UI inline
ns = Slint.load_source(<<~SLINT)
  export component App inherits Window {
    in-out property <string> greeting: "Hello, Ruby!";
    callback clicked();

    VerticalLayout {
      Text { text: root.greeting; }
      Button {
        text: "Click me";
        clicked => { root.clicked(); }
      }
    }
  }
SLINT

app = ns::App.new
app.on_clicked { app.greeting = "Clicked!" }
app.run
```

Or load from a `.slint` file:

```ruby
ns = Slint.load_file("ui/app.slint")
app = ns::App.new
app.run
```

## API

### Loading UI

| Method | Description |
|--------|-------------|
| `Slint.load_source(source, path = "inline.slint", **opts)` | Compile Slint source string |
| `Slint.load_file(path, **opts)` | Compile a `.slint` file |

Options: `style:` (e.g. `"fluent"`), `include_paths:` (array of search paths).

Both return a `Module` with component classes.

### Components

```ruby
ns = Slint.load_source(source)
app = ns::App.new                    # create instance
app = ns::App.new(label: "Hi")       # with initial property values

# Properties (kebab-case in Slint becomes snake_case in Ruby)
app.label                            # getter
app.label = "World"                  # setter

# Callbacks
app.on_clicked { |arg| puts arg }    # set handler (block)
app.on_clicked(proc { |arg| arg })   # set handler (proc)
app.invoke_clicked("test")           # invoke

# Window
app.show
app.hide
app.run                              # show + event loop + hide
```

### Globals

```ruby
# Access exported globals via method on the component instance
app.AppState.current_page            # get global property
app.AppState.current_page = "home"   # set global property
app.AppState.on_navigate { |p| ... } # global callback
app.AppState.invoke_navigate("about")
```

### ListModel

```ruby
model = Slint::ListModel.new([1, 2, 3])
model.push(4)
model << 5
model.row_count        # => 5
model.row_data(0)      # => 1.0
model.set_row_data(0, 10)
model.insert(1, 99)
model.remove(2)
model.clear
model.to_a             # => [10.0, 99.0, ...]
```

### Color & Brush

```ruby
color = Slint::Color.new(255, 0, 0)         # RGB
color = Slint::Color.rgba(255, 0, 0, 128)   # RGBA
color.red    # => 255
color.brighter(0.5)
color.darker(0.3)

brush = Slint::Brush.from_color(color)
brush.color  # => Slint::Color
```

### Image

```ruby
img = Slint::Image.load_from_path("icon.png")
img.width    # => 64
img.height   # => 64
img.path     # => "icon.png"
```

### Timer

```ruby
timer = Slint::Timer.new
timer.start_repeated(0.5, proc { puts "tick" })
timer.start_single_shot(1.0, proc { puts "once" })
timer.running?   # => true
timer.interval   # => 0.5
timer.stop
timer.restart
```

### Event Loop

```ruby
Slint.run_event_loop
Slint.quit_event_loop
Slint.invoke_from_event_loop(proc { ... })  # run code on the event loop thread
```

### Low-Level API

The `Slint::Compiler`, `Slint::CompilationResult`, `Slint::ComponentDefinition`, and `Slint::ComponentInstance` classes are available for advanced use:

```ruby
compiler = Slint::Compiler.new
compiler.style = "fluent"
compiler.include_paths = ["/path/to/includes"]

result = compiler.build_from_source(source, "app.slint")
raise result.diagnostics.map(&:to_s).join("\n") if result.has_errors?

definition = result.component("App")
instance = definition.create
instance.set_property("label", "Hello")
instance.get_property("label")  # => "Hello"
instance.set_callback("clicked", proc { |arg| "got: #{arg}" })
instance.invoke("clicked", ["test"])  # => "got: test"
instance.run
```

## Development

```sh
git clone https://github.com/masak1yu/slint-ruby
cd slint-ruby
bundle install
bundle exec rake compile
bundle exec rake test
```

## License

MIT
