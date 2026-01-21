# Performance Estimation - nwidgets

## Widgets

### Panel (src/widgets/panel/)
**Render Frequency**: 60 FPS (always visible)
**Performance Impact**: HIGH
- **mod.rs**: Main panel container, renders all modules
  - Allocations: 1 theme clone per frame
  - Optimizations: Memoization needed for static content
  
- **modules/workspaces.rs**: Workspace indicators
  - Allocations: Vec iteration per frame
  - Optimizations: ✅ Already optimal (small fixed list)
  
- **modules/active_window.rs**: Current window title
  - Allocations: ~~String clone per frame~~ ✅ OPTIMIZED
  - Optimizations: ✅ Cache title, update only on change (SharedString)
  
- **modules/datetime.rs**: Clock display
  - Allocations: ~~String formatting every second~~ ✅ OPTIMIZED
  - Optimizations: ✅ Cache formatted string, update only on minute change (60s interval)
  
- **modules/sink.rs**: Volume indicator
  - Allocations: Icon lookup per frame
  - Optimizations: ✅ Already optimal
  
- **modules/source.rs**: Microphone indicator
  - Allocations: Icon lookup per frame
  - Optimizations: ✅ Already optimal
  
- **modules/network.rs**: Network status
  - Allocations: Icon lookup per frame
  - Optimizations: ✅ Already optimal
  
- **modules/bluetooth.rs**: Bluetooth status
  - Allocations: Format string for device count
  - Optimizations: ✅ Already optimal
  
- **modules/systray.rs**: System tray icons
  - Allocations: Vec iteration + image decoding per icon
  - Optimizations: ⚠️ Cache decoded images
  
- **modules/mpris.rs**: Media player controls
  - Allocations: String clones for title/artist
  - Optimizations: ⚠️ Cache metadata, update only on change
  
- **modules/pomodoro.rs**: Pomodoro timer
  - Allocations: String formatting per second
  - Optimizations: ✅ Already optimal (updates only when active)

### Control Center (src/widgets/control_center/)
**Render Frequency**: On-demand (when open)
**Performance Impact**: MEDIUM
- **mod.rs**: Main container with render logic
  - Allocations: 1 theme clone, animation state
  - Optimizations: ✅ Lazy loading with .take() limits
  
- **audio.rs**: Audio sliders section
  - Allocations: 1 theme clone, icon lookups
  - Optimizations: ✅ Already optimal
  
- **quick_actions.rs**: Connectivity buttons
  - Allocations: State reads per button
  - Optimizations: ✅ Already optimal
  
- **notifications.rs**: Notifications list
  - Allocations: Vec iteration (max 5)
  - Optimizations: ✅ Lazy loading with .take(5)
  
- **details/sink.rs**: Sink device details
  - Allocations: Vec iteration (max 5 streams)
  - Optimizations: ✅ Lazy loading with .take(5)
  
- **details/source.rs**: Source device details
  - Allocations: Vec iteration (max 5 streams)
  - Optimizations: ✅ Lazy loading with .take(5)
  
- **details/bluetooth.rs**: Bluetooth devices
  - Allocations: Vec iteration (max 8 devices)
  - Optimizations: ✅ Lazy loading with .take(8)
  
- **details/network.rs**: Network + VPN
  - Allocations: Vec iteration (max 6 VPN)
  - Optimizations: ✅ Lazy loading with .take(6)
  
- **details/monitor.rs**: System monitor
  - Allocations: Vec iteration (max 7 disks), circular progress calculations
  - Optimizations: ✅ Lazy loading with .take(7)

### Launcher (src/widgets/launcher.rs)
**Render Frequency**: On-demand (when open)
**Performance Impact**: MEDIUM
- Allocations: Search results iteration, fuzzy matching
- Optimizations: ⚠️ Limit results to 10-15, debounce search

### OSD (src/widgets/osd.rs)
**Render Frequency**: On-demand (volume/brightness changes)
**Performance Impact**: LOW
- Allocations: Minimal, short-lived widget
- Optimizations: ✅ Already optimal

### Notifications (src/widgets/notifications.rs)
**Render Frequency**: On-demand (new notifications)
**Performance Impact**: LOW
- Allocations: Per notification rendering
- Optimizations: ✅ Already optimal

### Chat (src/widgets/chat.rs)
**Render Frequency**: On-demand (when open)
**Performance Impact**: HIGH (CEF browser)
- Allocations: Browser rendering, texture updates
- Optimizations: ⚠️ Offscreen rendering, texture caching

## Services

### High Frequency Updates (Performance Critical)

**audio.rs**: Audio state monitoring
- Update Frequency: ~100ms
- Allocations: State clones, stream lists
- Optimizations: ✅ Memoization cache (100ms TTL)

**system_monitor.rs**: CPU/GPU/RAM/Network stats
- Update Frequency: ~1000ms
- Allocations: Metrics collection, disk iteration
- Optimizations: ⚠️ Cache metrics, update only changed values

**hyprland.rs**: Workspace/window events
- Update Frequency: On events (variable)
- Allocations: JSON parsing, event handling
- Optimizations: ✅ Event-driven, no polling

**mpris.rs**: Media player state
- Update Frequency: On events (variable)
- Allocations: Metadata clones, DBus calls
- Optimizations: ⚠️ Cache metadata, diff updates

### Medium Frequency Updates

**network/**: Network state (wifi, ethernet, vpn)
- Update Frequency: ~2000ms
- Allocations: NetworkManager DBus calls, connection lists
- Optimizations: ✅ Event-driven + polling fallback

**bluetooth.rs**: Bluetooth devices
- Update Frequency: ~2000ms
- Allocations: Device list iteration, DBus calls
- Optimizations: ✅ Event-driven + polling fallback

**systray.rs**: System tray items
- Update Frequency: On events (variable)
- Allocations: Icon data, image decoding
- Optimizations: ⚠️ Cache decoded icons

**notifications.rs**: Notification management
- Update Frequency: On events (variable)
- Allocations: Notification storage
- Optimizations: ✅ Event-driven

### Low Frequency Updates

**launcher/**: Application launcher
- Update Frequency: On-demand
- Allocations: Desktop file parsing, fuzzy search
- Optimizations: ✅ Cached app list, incremental search

**pomodoro.rs**: Pomodoro timer
- Update Frequency: ~1000ms (when active)
- Allocations: Minimal
- Optimizations: ✅ Only updates when running

**control_center.rs**: Control center state
- Update Frequency: On-demand
- Allocations: Minimal
- Optimizations: ✅ State-only service

**osd.rs**: OSD state
- Update Frequency: On-demand
- Allocations: Minimal
- Optimizations: ✅ State-only service

**chat.rs**: Chat service
- Update Frequency: On-demand
- Allocations: Message storage
- Optimizations: ✅ Lazy loading

**cef/**: CEF browser integration
- Update Frequency: 60 FPS (when visible)
- Allocations: Browser rendering, texture updates
- Optimizations: ⚠️ Offscreen rendering, pause when hidden

## Priority Optimizations

### Critical (Panel - 60 FPS)
1. ~~**datetime.rs**: Cache formatted time, update only on minute change~~ ✅ DONE
2. ~~**active_window.rs**: Cache title, update only on window change~~ ✅ DONE
3. **systray.rs**: Cache decoded icon images
4. **mpris.rs**: Cache metadata, diff updates

### High (Control Center - On-demand)
1. **monitor.rs**: Cache metrics, update only changed values
2. **system_monitor.rs**: Optimize disk iteration

### Medium (Launcher - On-demand)
1. **launcher.rs**: Limit search results to 10-15
2. **fuzzy.rs**: Debounce search input

### Low (Chat - On-demand)
1. **cef/**: Pause rendering when hidden
2. **browser.rs**: Texture caching

## Memory Usage Estimates

- **Panel**: ~2-5 MB (always resident)
- **Control Center**: ~3-8 MB (when open)
- **Launcher**: ~5-10 MB (when open, includes app cache)
- **Chat**: ~50-100 MB (CEF browser)
- **Services**: ~10-20 MB total
- **Total**: ~70-143 MB (all widgets open)

## CPU Usage Estimates

- **Panel**: ~1-2% (60 FPS rendering)
- **Control Center**: ~2-5% (when open)
- **Launcher**: ~5-10% (fuzzy search)
- **Chat**: ~10-20% (CEF rendering)
- **Services**: ~1-3% (background monitoring)
- **Total**: ~19-40% (all active)

## Recommendations

1. ✅ **Lazy loading**: Already implemented with .take() limits
2. ✅ **Memoization**: Already implemented for audio state
3. ⚠️ **Icon caching**: Implement for systray
4. ⚠️ **Metadata caching**: Implement for mpris, active_window
5. ⚠️ **Time formatting**: Cache and update only on minute change
6. ⚠️ **Search debouncing**: Implement for launcher
7. ⚠️ **CEF optimization**: Pause when hidden
