# CEF + wgpu Optimizations for nwidgets

## Current State (Suboptimal)

**File**: `src/services/cef/init.rs` (lines 86-95)

```rust
// Force software rendering globally to avoid EGL conflicts with GPUI
log::info!("Forcing CEF software rendering (SwiftShader) for stability");
cmd.append_switch(Some(&"disable-gpu".into()));
cmd.append_switch(Some(&"disable-gpu-compositing".into()));
cmd.append_switch(Some(&"enable-unsafe-swiftshader".into()));
cmd.append_switch_with_value(
    Some(&"use-gl".into()),
    Some(&"swiftshader".into()),
);
```

**Problem**: CEF uses CPU-based SwiftShader rendering, then copies pixels to GPUI. This is slow and wastes GPU resources.

## Optimizations from tauri-apps/cef-rs OSR Example

### 1. Enable GPU Acceleration

**Reference**: `cef-rs/examples/osr/src/main.rs`

Remove software rendering flags and enable GPU:

```rust
// REMOVE these lines:
cmd.append_switch(Some(&"disable-gpu".into()));
cmd.append_switch(Some(&"disable-gpu-compositing".into()));
cmd.append_switch(Some(&"enable-unsafe-swiftshader".into()));
cmd.append_switch_with_value(Some(&"use-gl".into()), Some(&"swiftshader".into()));

// ADD these instead:
cmd.append_switch(Some(&"enable-gpu".into()));
cmd.append_switch(Some(&"enable-gpu-compositing".into()));
cmd.append_switch_with_value(
    Some(&"use-angle".into()),
    Some(&"vulkan".into()),  // Use Vulkan backend on Linux
);
```

### 2. Enable Shared Texture (Zero-Copy GPU Rendering)

**Reference**: `cef-rs/examples/osr/src/main.rs` (lines 280-285)

```rust
let window_info = WindowInfo {
    windowless_rendering_enabled: true as _,
    shared_texture_enabled: true as _,        // NEW: Enable shared textures
    external_begin_frame_enabled: true as _,  // NEW: External frame control
    ..Default::default()
};
```

**File to modify**: `src/services/cef/browser.rs` (around line 141)

### 3. Implement `on_accelerated_paint` Handler

**Reference**: `cef-rs/examples/osr/src/webrender.rs` (lines 140-200)

Add to `src/services/cef/handlers.rs`:

```rust
#[cfg(all(target_os = "linux", feature = "accelerated_osr"))]
fn on_accelerated_paint(
    &self,
    _browser: Option<&mut Browser>,
    type_: PaintElementType,
    _dirty_rects: Option<&[Rect]>,
    info: Option<&AcceleratedPaintInfo>,
) {
    let Some(info) = info else { return };
    
    use cef::osr_texture_import::shared_texture_handle::SharedTextureHandle;
    
    if type_ != PaintElementType::default() {
        return;
    }
    
    let shared_handle = SharedTextureHandle::new(info);
    if let SharedTextureHandle::Unsupported = shared_handle {
        log::error!("Platform does not support accelerated painting");
        return;
    }
    
    // Import CEF's GPU texture directly into wgpu
    match shared_handle.import_texture(&self.wgpu_device) {
        Ok(texture) => {
            // Update GPUI's render image with the imported texture
            // This is zero-copy - no CPU involvement!
            self.update_render_texture(texture);
        }
        Err(e) => {
            log::error!("Failed to import shared texture: {:?}", e);
        }
    }
}
```

### 4. Access wgpu Device from GPUI

**Challenge**: Need to get wgpu `Device` and `Queue` from GPUI context.

**Research needed**: Check if GPUI exposes wgpu device/queue in its public API.

Possible approaches:
- `cx.gpu_device()` or similar method
- Access via `Window` or `Context`
- May need to patch GPUI to expose wgpu internals

### 5. Feature Flag Configuration

Add to `Cargo.toml`:

```toml
[features]
default = ["accelerated_osr"]
accelerated_osr = []
```

## Expected Performance Gains

### Before (Current - Software Rendering)
1. CEF renders to CPU buffer (SwiftShader)
2. Copy CPU buffer → GPU texture (via GPUI)
3. GPUI renders texture to screen

**Bottleneck**: CPU rendering + CPU→GPU copy

### After (GPU Acceleration)
1. CEF renders directly to GPU texture (Vulkan)
2. Share GPU texture handle with GPUI (zero-copy)
3. GPUI renders shared texture to screen

**Benefits**:
- ✅ No CPU rendering overhead
- ✅ No CPU→GPU memory copy
- ✅ Lower latency (direct GPU→GPU)
- ✅ Better frame pacing (external_begin_frame_enabled)
- ✅ Reduced memory usage

**Estimated improvement**: 2-3x faster rendering, 50% less memory usage

## Implementation Steps

1. **Phase 1**: Enable GPU rendering (remove SwiftShader flags)
2. **Phase 2**: Enable shared textures in WindowInfo
3. **Phase 3**: Implement `on_accelerated_paint` handler
4. **Phase 4**: Integrate with GPUI's wgpu device
5. **Phase 5**: Test and benchmark

## Risks & Fallbacks

- If shared textures fail, fallback to `on_paint` (current CPU path)
- Add feature flag to disable acceleration if needed
- Test on AMD/NVIDIA/Intel GPUs

## References

- cef-rs OSR example: `/tmp/cef-rs/examples/osr/`
- CEF shared texture docs: https://bitbucket.org/chromiumembedded/cef/wiki/GeneralUsage#markdown-header-off-screen-rendering
- wgpu texture import: `cef::osr_texture_import::shared_texture_handle`
