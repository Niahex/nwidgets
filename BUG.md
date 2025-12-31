# Bug Report: Slider Fluidity Issues in Control Center

## Problem
The volume and microphone sliders in the control center experience fluidity/performance issues during interaction.

## Context
- **Component**: `src/widgets/control_center.rs`
- **Slider Implementation**: Custom `Slider` component from `src/components/slider.rs`
- **Audio Backend**: PipeWire via `wpctl` commands (async with `gpui_tokio`)

## Current Implementation

### Volume Control Flow
1. User drags slider → `SliderEvent::Change` emitted
2. Visual update happens immediately (`this.last_volume = new_volume`)
3. Throttled `wpctl` command (30ms delay between calls)
4. Async execution via `gpui_tokio::Tokio::spawn`

### Throttling Strategy
```rust
if this.last_volume_update.map(|last| now.duration_since(last) >= Duration::from_millis(30)).unwrap_or(true) {
    this.last_volume_update = Some(now);
    this.audio.update(cx, |audio, cx| {
        audio.set_sink_volume(new_volume, cx);
    });
}
```

## Attempted Solutions

### 1. ✅ Replaced Scroll Wheel with Slider
- **Before**: Manual scroll wheel handling with `on_scroll_wheel`
- **After**: Proper `Slider` component with drag support
- **Result**: Better UX but still performance issues

### 2. ✅ Async wpctl Commands
- **Before**: Blocking `std::process::Command`
- **After**: `tokio::process::Command` with `gpui_tokio`
- **Result**: Non-blocking but still laggy

### 3. ✅ Throttling
- Added 30ms throttle between wpctl calls
- Visual updates happen immediately
- **Result**: Reduced command overhead but slider still not smooth

### 4. ❌ libpulse-binding
- Attempted direct PulseAudio/PipeWire API
- **Result**: Compilation issues, reverted to wpctl

## Observations

### Potential Causes
1. **Slider Component**: The custom slider might have rendering overhead
2. **Event Frequency**: Drag events might be too frequent even with throttling
3. **Context Updates**: `cx.notify()` on every event might trigger unnecessary re-renders
4. **Audio Service**: The update chain `audio.update(cx, |audio, cx| ...)` might be slow

### Performance Characteristics
- Visual updates are instant (local state)
- wpctl calls are throttled (30ms)
- Async execution prevents blocking
- But slider still feels laggy during drag

## Next Steps

### To Investigate
- [ ] Profile slider render performance
- [ ] Measure actual event frequency during drag
- [ ] Check if `cx.notify()` is causing re-render cascade
- [ ] Compare with other GPUI slider implementations (Zed, gpui-component)
- [ ] Test with different throttle values (50ms, 100ms)

### Alternative Approaches
1. **Debouncing**: Only update after drag ends (like scroll wheel)
2. **Separate Visual State**: Decouple visual slider from audio state completely
3. **Native Slider**: Use a simpler div-based slider without complex component
4. **Batch Updates**: Collect multiple events and apply in batches

## Related Files
- `src/widgets/control_center.rs` - Main implementation
- `src/components/slider.rs` - Slider component
- `src/services/audio.rs` - Audio service with wpctl
- `Cargo.toml` - Dependencies (gpui_tokio, tokio)

## Environment
- **OS**: Linux (NixOS)
- **Audio**: PipeWire
- **Framework**: GPUI v0.216.1
- **Build**: Nix flake with development shell
