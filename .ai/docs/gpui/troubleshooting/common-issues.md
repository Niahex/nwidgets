# Common Issues

Solutions to frequently encountered problems in GPUI applications.

## Application Issues

### Window Not Appearing
**Problem**: Window opens but is not visible.

**Solution**:
```rust
Application::new().run(|cx: &mut App| {
    cx.open_window(/* ... */);
    cx.activate(true); // Required!
});
```

### High CPU Usage
**Problem**: Application consuming excessive CPU.

**Solutions**:
1. **Avoid Unnecessary Re-renders**:
```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Cache expensive computations
        if self.needs_update {
            self.cached_result = self.expensive_computation();
            self.needs_update = false;
        }
        div().child(&self.cached_result)
    }
}
```

2. **Use Efficient Lists**:
```rust
// Use uniform_list for large datasets
uniform_list(/* ... */)
```

### Memory Leaks
**Problem**: Memory usage grows over time.

**Solutions**:
1. **Clean Up Subscriptions**:
```rust
struct MyView {
    _subscription: Subscription, // Automatically cleaned up
}
```

2. **Avoid Circular References**:
```rust
use std::rc::Weak;

struct Child {
    parent: Weak<Parent>, // Use weak references
}
```

## Rendering Issues

### Blurry Text
**Problem**: Text appears blurry or pixelated.

**Solutions**:
1. **Check DPI Settings**:
```bash
# Linux
echo $GDK_SCALE

# Ensure proper scaling
export GDK_SCALE=2
```

2. **Font Configuration**:
```rust
text("Hello")
    .size(px(16.)) // Use appropriate font sizes
```

### Layout Problems
**Problem**: Elements not positioned correctly.

**Solutions**:
1. **Debug Layout**:
```rust
div()
    .border_1() // Add borders to see element bounds
    .border_color(rgb(0xff0000))
```

2. **Check Flexbox Properties**:
```rust
div()
    .flex()
    .flex_col() // Ensure correct direction
    .justify_center()
    .items_center()
```

## Platform-Specific Issues

### NixOS Issues
**Problem**: Application doesn't work on NixOS.

**Solutions**:
1. **Check Wayland Support**:
```bash
echo $WAYLAND_DISPLAY
```

2. **Add Dependencies**:
```nix
# shell.nix
buildInputs = with pkgs; [
  wayland libxkbcommon vulkan-loader
];
```

### Graphics Issues
**Problem**: Graphics rendering problems.

**Solutions**:
1. **Check Vulkan Support**:
```bash
vulkaninfo
```

2. **NixOS Graphics Configuration**:
```nix
# configuration.nix
hardware.opengl = {
  enable = true;
  driSupport = true;
};
```

## Build Issues

### Compilation Errors
**Problem**: Code doesn't compile.

**Common Solutions**:
1. **Update Rust**:
```bash
rustup update
```

2. **Clean Build**:
```bash
cargo clean
cargo build
```

3. **Check Dependencies**:
```toml
[dependencies]
gpui = "0.2.2" # Use exact version
```

### Linking Errors
**Problem**: Linker errors during build.

**Solutions**:
1. **NixOS Development Environment**:
```nix
# shell.nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    pkg-config
    cmake
    wayland
    libxkbcommon
  ];
}
```

2. **Check Feature Flags**:
```toml
gpui = { version = "0.2.2", features = ["wayland"] }
```

## Performance Issues

### Slow Rendering
**Problem**: UI feels sluggish.

**Solutions**:
1. **Profile Performance**:
```rust
use gpui::profiler;

profiler::scope!("expensive_operation");
```

2. **Optimize Render Methods**:
```rust
// Avoid complex computations in render
impl Render for MyView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Pre-compute in update methods
        div().child(&self.precomputed_content)
    }
}
```

### Memory Usage
**Problem**: High memory consumption.

**Solutions**:
1. **Monitor Memory**:
```bash
htop
```

2. **Use Weak References**:
```rust
use std::rc::Weak;
```

## Debugging Tips

### Enable Debug Logging
```rust
use log::LevelFilter;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();
}
```

### Visual Debugging
```rust
// Add visual debugging aids
div()
    .border_1()
    .border_color(rgb(0xff0000)) // Red border
    .bg(rgba(0xff000020))        // Semi-transparent background
```

### Panic Handling
```rust
use std::panic;

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic: {:?}", panic_info);
    }));
}
```
