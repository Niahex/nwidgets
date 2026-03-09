# CEF GPU Acceleration - Platform Layer Implementation Plan

## Architecture Overview

```
nwidgets (application)
    ↓ CEF AcceleratedPaintInfo
gpui_linux::cef_gpu (new module)
    ↓ import_cef_shared_texture()
gpui_wgpu::WgpuRenderer (internal access)
    ↓ wgpu::Device
wgpu-hal::Vulkan (low-level)
    ↓ texture_from_raw()
Vulkan shared texture (zero-copy)
```

## Implementation Steps

### Phase 1: Create CEF GPU Module in gpui_linux

**File**: `crates/gpui_linux/src/linux/cef_gpu.rs`

```rust
use std::sync::Arc;
use gpui_wgpu::wgpu;

pub struct CefTextureHandle {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

#[cfg(target_os = "linux")]
pub fn import_cef_shared_texture(
    device: &wgpu::Device,
    fd: i32,  // DMA-BUF file descriptor from CEF
    width: u32,
    height: u32,
) -> anyhow::Result<CefTextureHandle> {
    // Use wgpu-hal to import Vulkan external memory
    // This requires unsafe code and platform-specific APIs
    todo!("Implement Vulkan DMA-BUF import")
}
```

**Challenges**:
- wgpu-hal APIs are unstable and may change
- Requires VK_EXT_external_memory_dma_buf extension
- Platform-specific (Linux only)

### Phase 2: Expose in WaylandWindow/X11Window

**File**: `crates/gpui_linux/src/linux/wayland/window.rs`

Add method to WaylandWindow:

```rust
impl WaylandWindow {
    pub fn import_cef_texture(
        &self,
        fd: i32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<cef_gpu::CefTextureHandle> {
        let renderer = &self.borrow().renderer;
        let device = renderer.device()?;  // Internal access
        cef_gpu::import_cef_shared_texture(&device, fd, width, height)
    }
}
```

**Problem**: This requires accessing `renderer.device()` which is private.

**Solution**: Add internal method in WgpuRenderer:

```rust
// In gpui_wgpu/src/wgpu_renderer.rs
impl WgpuRenderer {
    pub(crate) fn device_for_platform(&self) -> &wgpu::Device {
        &self.resources().device
    }
}
```

### Phase 3: Implement in nwidgets CEF Handler

**File**: `nwidgets/src/services/cef/handlers.rs`

```rust
#[cfg(feature = "accelerated_osr")]
fn on_accelerated_paint(
    &self,
    _browser: Option<&mut Browser>,
    type_: PaintElementType,
    _dirty_rects: Option<&[Rect]>,
    info: Option<&AcceleratedPaintInfo>,
) {
    let Some(info) = info else { return };
    
    if type_ != PaintElementType::default() {
        return;
    }
    
    // Get shared texture handle from CEF
    let shared_handle = SharedTextureHandle::new(info);
    
    // Import into GPUI via platform layer
    // This requires passing window handle to CEF handler
    let texture = self.window.import_cef_texture(
        shared_handle.fd(),
        info.width,
        info.height,
    )?;
    
    // Update render image
    self.update_texture(texture);
}
```

### Phase 4: Enable GPU Rendering in CEF

**File**: `nwidgets/src/services/cef/init.rs`

Remove SwiftShader flags:

```rust
// REMOVE:
cmd.append_switch(Some(&"disable-gpu".into()));
cmd.append_switch(Some(&"disable-gpu-compositing".into()));
cmd.append_switch(Some(&"enable-unsafe-swiftshader".into()));

// ADD:
cmd.append_switch(Some(&"enable-gpu".into()));
cmd.append_switch(Some(&"enable-gpu-compositing".into()));
cmd.append_switch_with_value(
    Some(&"use-angle".into()),
    Some(&"vulkan".into()),
);
```

### Phase 5: Enable Shared Textures

**File**: `nwidgets/src/services/cef/browser.rs`

```rust
let window_info = WindowInfo {
    windowless_rendering_enabled: 1,
    shared_texture_enabled: 1,        // NEW
    external_begin_frame_enabled: 1,  // NEW
    ..Default::default()
};
```

## Risks & Challenges

### Technical Risks

1. **wgpu-hal Instability**
   - wgpu-hal APIs are not stable
   - May break with wgpu updates
   - Requires unsafe code

2. **Platform-Specific Code**
   - Linux-only (Vulkan DMA-BUF)
   - Different code paths for AMD/NVIDIA/Intel
   - Requires VK_EXT_external_memory_dma_buf

3. **CEF Compatibility**
   - CEF shared texture format may not match wgpu expectations
   - Synchronization issues between CEF and GPUI render loops
   - Memory management complexity

4. **Testing Complexity**
   - Requires testing on multiple GPU vendors
   - Hard to debug GPU-level issues
   - May cause driver crashes

### Alternative: Hybrid Approach

Keep CPU rendering as fallback:

```rust
fn on_accelerated_paint(&self, info: &AcceleratedPaintInfo) {
    match self.try_gpu_import(info) {
        Ok(texture) => {
            log::info!("Using GPU-accelerated CEF rendering");
            self.use_gpu_texture(texture);
        }
        Err(e) => {
            log::warn!("GPU import failed, falling back to CPU: {}", e);
            self.fallback_to_cpu_paint();
        }
    }
}
```

## Recommended Next Steps

### Option A: Wait for GPUI Official Support
- Wait for GPUI to provide official GPU context access
- Less risky, more maintainable
- May take months

### Option B: Implement with Feature Flag
- Add `accelerated-cef` feature flag
- Implement behind feature flag
- Document as experimental
- Provide CPU fallback

### Option C: Keep Current Approach
- SwiftShader works reliably
- Performance is acceptable for most use cases
- Focus on other optimizations (memory, startup time)

## Performance Expectations

### Current (SwiftShader CPU):
- Rendering: ~16ms per frame (60 FPS)
- Memory: ~200MB for textures
- CPU usage: ~15-20%

### Expected (GPU Accelerated):
- Rendering: ~5ms per frame (200 FPS)
- Memory: ~100MB (shared textures)
- CPU usage: ~5%
- GPU usage: ~10%

### Gains:
- 3x faster rendering
- 50% less memory
- 70% less CPU usage

## Conclusion

The Platform Layer approach is technically feasible but requires:
- Significant unsafe code
- Platform-specific implementations
- Extensive testing
- Maintenance burden

**Recommendation**: Document the approach and implement only if:
1. Performance becomes a critical bottleneck
2. Resources available for thorough testing
3. Willing to maintain platform-specific code

For now, focus on:
- Fixing the zbus/tokio issue (✅ Done)
- Implementing layer shell features (✅ Done)
- Optimizing other aspects of nwidgets
