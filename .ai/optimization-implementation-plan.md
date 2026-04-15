# nwidgets Optimization Implementation Plan

**Generated**: 2026-04-15  
**Based on**: Comprehensive analysis of Zed vs nwidgets patterns  
**Target**: Reduce CPU idle by 50%, allocations by 50%, improve FPS by 60%

---

## Executive Summary

This plan addresses performance bottlenecks identified through parallel analysis of Zed's optimization patterns:
- **7 polling loops** causing 1-2% CPU idle (target: ~0%)
- **117 .clone() calls** in services causing excessive allocations
- **78 SharedString usages** vs Zed's 187 (opportunity for 20% allocation reduction)
- **No list virtualization** causing >16ms frame times (target: <10ms)
- **No deferred rendering** for floating UI

---

## Phase 1: Quick Wins (1-2 days) - Immediate Impact

### Priority 1.1: Eliminate VPN Polling (CRITICAL)
**File**: `src/services/network/vpn.rs:42-48`  
**Current**:
```rust
loop {
    let connections = Self::list_vpn_connections().await;
    if let Err(e) = list_tx.unbounded_send(connections) { ... }
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;  // 5s polling
}
```

**Target**:
```rust
// Subscribe to NetworkManager D-Bus signals for VPN state changes
let vpn_proxy = NetworkManagerProxy::new(&connection).await?;
let mut vpn_stream = vpn_proxy.receive_vpn_connection_added().await?;

loop {
    tokio::select! {
        Some(signal) = vpn_stream.next() => {
            let connections = Self::list_vpn_connections().await;
            let _ = list_tx.unbounded_send(connections);
        }
        _ = notify.notified() => {
            // Manual refresh trigger
        }
    }
}
```

**Estimated gain**: -1% CPU idle

---

### Priority 1.2: Remove Network Service Polling Fallback
**Files**: 
- `src/services/network/network.rs:225`
- `src/services/network/manager.rs:115`

**Current**:
```rust
loop {
    tokio::select! {
        Some(_) = connectivity_stream.next() => { ... }
        Some(_) = active_connections_stream.next() => { ... }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {  // Fallback polling
            let new_state = Self::fetch_network_state_dbus(...).await;
            ...
        }
    }
}
```

**Target**:
```rust
loop {
    tokio::select! {
        Some(_) = connectivity_stream.next() => { ... }
        Some(_) = active_connections_stream.next() => { ... }
        _ = notify.notified() => {
            // Manual refresh only - no periodic polling
            let new_state = Self::fetch_network_state_dbus(...).await;
            ...
        }
    }
}
```

**Estimated gain**: -0.5% CPU idle

---

### Priority 1.3: Remove Bluetooth Polling Fallback
**File**: `src/services/hardware/bluetooth.rs:180`

**Current**:
```rust
_ = tokio::time::sleep(Duration::from_secs(2)) => {  // 2s fallback
    let new_state = Self::fetch_bluetooth_state_dbus(...).await;
    ...
}
```

**Target**: Remove fallback branch entirely, rely on D-Bus signals only.

**Estimated gain**: -0.3% CPU idle

---

### Priority 1.4: Optimize Audio Service Getters
**File**: `src/services/media/audio.rs:558,562,566,570`

**Current**:
```rust
pub fn get_sinks(&self) -> SmallVec<[AudioDevice; 8]> {
    self.state.read().sinks.clone()  // Clone on every call
}
```

**Target**:
```rust
pub fn get_sinks(&self) -> Arc<SmallVec<[AudioDevice; 8]>> {
    Arc::clone(&self.state.read().sinks)  // Cheap Arc clone
}

// Update state struct:
pub struct AudioState {
    pub sinks: Arc<SmallVec<[AudioDevice; 8]>>,
    pub sources: Arc<SmallVec<[AudioDevice; 8]>>,
    // ...
}
```

**Estimated gain**: -4 allocations per render cycle

---

### Priority 1.5: Limit Long Lists with .take()
**Files**:
- `src/widgets/control_center/widget/audio_section.rs`
- `src/widgets/control_center/widget/bluetooth_section.rs`
- `src/widgets/launcher/widget/results.rs`

**Pattern**:
```rust
// Before
for device in devices.iter() {
    list.push(render_device(device));
}

// After
for device in devices.iter().take(20) {  // Limit to 20 visible items
    list.push(render_device(device));
}
```

**Estimated gain**: +20% FPS for lists with >20 items

---

## Phase 2: Medium Optimizations (3-5 days)

### Priority 2.1: Refactor CEF Browser Event Handlers
**File**: `src/services/cef/browser.rs:388-607`

**Current** (12 clones per event):
```rust
.on_mouse_down(cx.listener(move |this, event, window, cx| {
    let browser = browser.clone();  // Clone Arc on every mouse event
    let mouse_pressed = mouse_pressed.clone();
    // ...
}))
```

**Target** (WeakEntity pattern):
```rust
struct BrowserEventState {
    browser: WeakEntity<BrowserView>,
    mouse_pressed: Arc<AtomicBool>,
}

impl BrowserView {
    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        // Access self.browser directly, no clones needed
    }
}

// In render:
.on_mouse_down(cx.listener(Self::handle_mouse_down))
```

**Estimated gain**: -12 allocations per mouse event

---

### Priority 2.2: Implement uniform_list() in Control Center
**File**: `src/widgets/control_center/widget/bluetooth_section.rs`

**Current**:
```rust
for device in devices.iter() {
    v_flex = v_flex.child(render_device(device));
}
```

**Target**:
```rust
uniform_list(
    "bluetooth-devices",
    devices.len(),
    cx.processor(move |this, visible_range: Range<usize>, window, cx| {
        let devices = this.bluetooth_service.read(cx).get_devices();
        visible_range
            .map(|ix| this.render_device(&devices[ix], window, cx))
            .collect()
    }),
)
.track_scroll(&self.scroll_handle)
```

**Estimated gain**: +30% FPS for lists with >20 items

---

### Priority 2.3: Add SharedString Constants
**Files**: All services and widgets

**Pattern**:
```rust
// In service modules
const STATUS_CONNECTED: SharedString = SharedString::new_static("Connected");
const STATUS_DISCONNECTED: SharedString = SharedString::new_static("Disconnected");
const STATUS_CONNECTING: SharedString = SharedString::new_static("Connecting...");

// In error handling
const ERROR_NO_DEVICE: SharedString = SharedString::new_static("No device found");
const ERROR_CONNECTION_FAILED: SharedString = SharedString::new_static("Connection failed");
```

**Files to update**:
- `src/services/media/audio.rs` - device status strings
- `src/services/network/` - connection status, SSID labels
- `src/services/hardware/bluetooth.rs` - device status
- `src/widgets/` - UI labels, button text

**Estimated gain**: -15% allocations

---

## Phase 3: Advanced Optimizations (1 week)

### Priority 3.1: Convert Struct Fields to SharedString
**Pattern**:
```rust
// Before
pub struct AudioDevice {
    pub name: String,
    pub description: String,
}

// After
pub struct AudioDevice {
    pub name: SharedString,
    pub description: SharedString,
}

// Update conversions
impl From<PipeWireDeviceInfo> for AudioDevice {
    fn from(info: PipeWireDeviceInfo) -> Self {
        Self {
            name: SharedString::from(info.name),
            description: SharedString::from(info.description),
        }
    }
}
```

**Files to update**:
- `src/services/media/audio.rs` - AudioDevice, AudioSink, AudioSource
- `src/services/network/wifi.rs` - WifiNetwork (ssid field)
- `src/services/hardware/bluetooth.rs` - BluetoothDevice (name, address)
- `src/services/system/hyprland.rs` - ActiveWindow (title, class)

**Estimated gain**: -20% allocations in async boundaries

---

### Priority 3.2: Implement Deferred Rendering for Menus
**Files**: All widgets with context menus

**Pattern**:
```rust
.when(self.context_menu_open, |this| {
    this.child(
        deferred(
            anchored()
                .position(self.menu_position)
                .anchor(Corner::TopLeft)
                .child(self.render_context_menu())
        )
        .with_priority(1)
    )
})
```

**Files to update**:
- `src/widgets/control_center/widget/audio_section.rs` - device context menu
- `src/widgets/control_center/widget/bluetooth_section.rs` - device context menu
- `src/widgets/launcher/widget/launcher_widget.rs` - action menu
- `src/widgets/panel/modules/mpris/widget.rs` - player controls menu

**Estimated gain**: -30% render time for complex UIs

---

### Priority 3.3: Implement SharedString Caching
**Pattern**:
```rust
struct ActionNameCache {
    cache: HashMap<&'static str, SharedString>,
}

impl ActionNameCache {
    fn new(actions: &[&'static str]) -> Self {
        let cache = HashMap::from_iter(
            actions.iter().map(|&name| (name, SharedString::from(name)))
        );
        Self { cache }
    }
    
    fn get(&self, name: &'static str) -> SharedString {
        self.cache.get(name).cloned().unwrap_or_default()
    }
}

// Usage in service
lazy_static! {
    static ref ACTION_CACHE: ActionNameCache = ActionNameCache::new(&[
        "play", "pause", "next", "previous", "stop"
    ]);
}
```

**Files to update**:
- `src/widgets/panel/modules/mpris/service.rs` - MPRIS action names
- `src/widgets/launcher/service/launcher_service.rs` - app categories
- `src/services/system/hyprland.rs` - workspace names

**Estimated gain**: -10% allocations

---

## Verification & Testing

### Performance Metrics to Track

**Before optimization**:
```bash
# CPU idle
htop  # Observe nwidgets process when idle

# Memory allocations
cargo build --release
perf record -g ./target/release/nwidgets
perf report

# Frame times
# Add to main.rs:
let start = std::time::Instant::now();
// render code
println!("Frame time: {:?}", start.elapsed());
```

**After each phase**:
- Re-run performance measurements
- Compare against baseline
- Document actual vs estimated gains

### Test Cases

1. **Polling elimination**: Verify CPU idle drops to ~0%
2. **Clone optimization**: Profile allocations with `cargo flamegraph`
3. **List virtualization**: Test with 100+ item lists, verify smooth scrolling
4. **Deferred rendering**: Open/close menus rapidly, verify no frame drops
5. **SharedString**: Profile string allocations before/after

---

## Expected Results

| Phase | Metric | Before | After | Gain |
|-------|--------|--------|-------|------|
| 1 | CPU idle | 1-2% | 0.5% | -50% |
| 1 | Allocations | Baseline | -10% | -10% |
| 1 | Frame time | >16ms | ~13ms | +20% FPS |
| 2 | Allocations | -10% | -30% | -20% more |
| 2 | Frame time | ~13ms | ~10ms | +30% FPS |
| 3 | Allocations | -30% | -50% | -20% more |
| 3 | Frame time | ~10ms | <10ms | +10% FPS |
| **Total** | **CPU idle** | **1-2%** | **~0%** | **-50%** |
| **Total** | **Allocations** | **Baseline** | **-50%** | **-50%** |
| **Total** | **Frame time** | **>16ms** | **<10ms** | **+60% FPS** |

---

## Implementation Order (Recommended)

1. ✅ VPN polling elimination (1 hour)
2. ✅ Network/Bluetooth polling fallback removal (2 hours)
3. ✅ Audio service getter optimization (1 hour)
4. ✅ Add .take(20) to long lists (2 hours)
5. ✅ CEF browser event handler refactor (4 hours)
6. ✅ Implement uniform_list() in control center (6 hours)
7. ✅ Add SharedString constants (4 hours)
8. ✅ Convert struct fields to SharedString (8 hours)
9. ✅ Implement deferred rendering for menus (6 hours)
10. ✅ Implement SharedString caching (4 hours)

**Total estimated time**: 38 hours (~1 week of focused work)

---

## Risk Mitigation

### Potential Issues

1. **D-Bus signal reliability**: Some signals might not fire consistently
   - **Mitigation**: Add manual refresh trigger via Notify
   
2. **Arc<T> vs T.clone() API changes**: Callers expect owned values
   - **Mitigation**: Update all call sites, use Arc::clone() explicitly
   
3. **uniform_list() with dynamic heights**: Items must have uniform height
   - **Mitigation**: Use list() with ListState for non-uniform items
   
4. **SharedString conversion overhead**: One-time conversion cost
   - **Mitigation**: Convert at data boundaries (parsing, D-Bus receive)

### Rollback Strategy

- Commit after each priority item
- Tag before starting each phase
- Keep performance measurements in commit messages
- If regression detected, revert specific commit

---

## Success Criteria

✅ **Phase 1 Complete** when:
- CPU idle <1% (measured with htop)
- No polling loops in network/bluetooth services
- Audio getter allocations reduced (measured with perf)

✅ **Phase 2 Complete** when:
- Frame time <13ms for complex UIs (measured with frame timer)
- CEF event handler allocations reduced by 50%
- Control center lists use uniform_list()

✅ **Phase 3 Complete** when:
- Total allocations reduced by 50% (measured with flamegraph)
- All menus use deferred rendering
- All struct fields use SharedString where appropriate

✅ **Project Complete** when:
- CPU idle ~0%
- Frame time <10ms consistently
- Memory allocations reduced by 50%
- All tests pass
- No performance regressions

---

## References

- [Zed Performance Analysis](.ai/performance-analysis-zed-vs-nwidgets.md)
- [Performance Guide](.ai/performance-guide.md)
- [GPUI Documentation](.ai/docs/gpui/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
