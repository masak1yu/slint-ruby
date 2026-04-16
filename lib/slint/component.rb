module Slint
  class Component
    attr_reader :__instance__

    def show
      __instance__.show
    end

    def hide
      __instance__.hide
    end

    def run
      show
      Slint.run_event_loop
      hide
    end
  end

  class << self
    def load_file(path, **opts)
      compiler = Slint::Compiler.new
      compiler.style = opts[:style] if opts[:style]
      compiler.include_paths = opts[:include_paths] if opts[:include_paths]

      result = compiler.build_from_path(path.to_s)

      if result.has_errors?
        messages = result.diagnostics.select { |d| d.level == "error" }.map(&:to_s)
        raise RuntimeError, "Slint compilation errors:\n#{messages.join("\n")}"
      end

      ns = Module.new

      result.component_names.each do |comp_name|
        compdef = result.component(comp_name)
        cls = _build_class(compdef)
        ns.const_set(comp_name.to_sym, cls)
      end

      ns
    end

    def load_source(source, path = "inline.slint", **opts)
      compiler = Slint::Compiler.new
      compiler.style = opts[:style] if opts[:style]
      compiler.include_paths = opts[:include_paths] if opts[:include_paths]

      result = compiler.build_from_source(source, path)

      if result.has_errors?
        messages = result.diagnostics.select { |d| d.level == "error" }.map(&:to_s)
        raise RuntimeError, "Slint compilation errors:\n#{messages.join("\n")}"
      end

      ns = Module.new

      result.component_names.each do |comp_name|
        compdef = result.component(comp_name)
        cls = _build_class(compdef)
        ns.const_set(comp_name.to_sym, cls)
      end

      ns
    end

    private

    def _normalize_prop(name)
      name.tr("-", "_")
    end

    def _build_class(compdef)
      cls = Class.new(Component)

      # Store compdef for access in initialize
      cls.instance_variable_set(:@__compdef__, compdef)

      cls.define_method(:initialize) do |**kwargs|
        @__instance__ = self.class.instance_variable_get(:@__compdef__).create
        kwargs.each { |prop, val| send(:"#{Slint.send(:_normalize_prop, prop.to_s)}=", val) }
      end

      # Properties: getter + setter
      compdef.properties.each do |prop_name, _type|
        ruby_name = _normalize_prop(prop_name)

        cls.define_method(ruby_name) do
          __instance__.get_property(prop_name)
        end

        cls.define_method(:"#{ruby_name}=") do |val|
          __instance__.set_property(prop_name, val)
        end
      end

      # Callbacks: invoke + set
      compdef.callbacks.each do |cb_name|
        ruby_name = _normalize_prop(cb_name)

        # Invoking the callback
        cls.define_method(:"invoke_#{ruby_name}") do |*args|
          __instance__.invoke(cb_name, args)
        end

        # Setting the callback handler
        cls.define_method(:"on_#{ruby_name}") do |callable = nil, &block|
          handler = callable || block
          raise ArgumentError, "Expected a callable or block" unless handler
          __instance__.set_callback(cb_name, handler)
        end
      end

      # Functions: invoke
      compdef.functions.each do |fn_name|
        ruby_name = _normalize_prop(fn_name)

        cls.define_method(ruby_name) do |*args|
          __instance__.invoke(fn_name, args)
        end
      end

      # Globals: nested accessor
      compdef.globals.each do |global_name|
        global_cls = _build_global_class(compdef, global_name)
        ruby_name = _normalize_prop(global_name)

        cls.define_method(ruby_name) do
          wrapper = global_cls.new
          wrapper.instance_variable_set(:@__instance__, __instance__)
          wrapper
        end
      end

      cls
    end

    def _build_global_class(compdef, global_name)
      cls = Class.new

      # Properties
      if (props = compdef.global_properties(global_name))
        props.each do |prop_name, _type|
          ruby_name = _normalize_prop(prop_name)

          cls.define_method(ruby_name) do
            @__instance__.get_global_property(global_name, prop_name)
          end

          cls.define_method(:"#{ruby_name}=") do |val|
            @__instance__.set_global_property(global_name, prop_name, val)
          end
        end
      end

      # Callbacks
      if (cbs = compdef.global_callbacks(global_name))
        cbs.each do |cb_name|
          ruby_name = _normalize_prop(cb_name)

          cls.define_method(:"invoke_#{ruby_name}") do |*args|
            @__instance__.invoke_global(global_name, cb_name, args)
          end

          cls.define_method(:"on_#{ruby_name}") do |callable = nil, &block|
            handler = callable || block
            raise ArgumentError, "Expected a callable or block" unless handler
            @__instance__.set_global_callback(global_name, cb_name, handler)
          end
        end
      end

      # Functions
      if (fns = compdef.global_functions(global_name))
        fns.each do |fn_name|
          ruby_name = _normalize_prop(fn_name)

          cls.define_method(ruby_name) do |*args|
            @__instance__.invoke_global(global_name, fn_name, args)
          end
        end
      end

      cls
    end
  end
end
