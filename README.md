# nwidgets

A high-performance Wayland widget system built with GPUI, featuring a customizable panel, launcher, notifications, and more.

## Features

### 🎨 Panel
- **Top bar** with workspace indicators, system tray, and status modules
- **Active window** title display
- **MPRIS** media controls with album art
- **System monitor** (CPU, RAM, temperature)
- **Audio/Bluetooth** controls
- **Network** status
- **Date/Time** display
- **Pomodoro** timer

### 🚀 Launcher
- **Fast application launcher** with fuzzy search
- **Calculator** mode (type `=` to calculate)
- **Process manager** (type `ps` to list/kill processes)
- **Clipboard history** integration

### 💬 Chat
- **Embedded browser** for Gemini AI chat
- **Custom Nord theme** CSS injection
- **Fullscreen-aware** (auto-hide in fullscreen)
- **Persistent URL** state

### 🔔 Notifications
- **Native Wayland notifications**
- **Auto-dismiss** after timeout
- **Click to dismiss**

### 🎛️ Control Center
- **Quick actions** (WiFi, Bluetooth, DND, etc.)
- **Audio mixer** with per-app volume control
- **Bluetooth device manager**
- **Network manager**
- **Notification settings**

### 📊 OSD (On-Screen Display)
- **Volume** indicator
- **Brightness** indicator
- **Microphone mute** indicator

## Architecture

### Performance Optimizations
- **Event-driven** architecture (no polling)
- **Deferred rendering** for complex views
- **Lazy loading** for lists
- **SharedString caching** for UI strings
- **On-demand monitoring** (services only active when needed)

### Optimization Techniques
1. **Event-driven updates** - No polling loops
2. **State comparison** - Only emit events on actual changes
3. **Deferred rendering** - Complex views render asynchronously
4. **Lazy loading** - Lists only render visible items
5. **String caching** - SharedString for all UI text
6. **On-demand services** - Services sleep when not needed

See `.ai/performance-guide.md` for detailed optimization patterns.

### Services
All services are global singletons with event-driven updates:
- `HyprlandService` - Workspace and window management
- `AudioService` - PulseAudio integration
- `BluetoothService` - BlueZ D-Bus integration
- `NetworkService` - NetworkManager integration
- `MprisService` - Media player control
- `SystemMonitorService` - CPU/RAM/Temp monitoring
- `NotificationService` - Freedesktop notifications
- `CefService` - Chromium Embedded Framework
- `ClipboardMonitor` - Clipboard history

## Installation

### NixOS (Flake)
```nix
{
  inputs = {
    nwidgets.url = "github:yourusername/nwidgets";
  };

  outputs = { self, nixpkgs, nwidgets }: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      modules = [
        nwidgets.nixosModules.default
        {
          programs.nwidgets.enable = true;
        }
      ];
    };
  };
}
```

## Configuration

### Theme
Edit `src/theme.rs` to customize colors (Nord Dark theme by default).

### Panel Modules
Edit `src/widgets/panel/mod.rs` to add/remove modules.

## Project Structure

```
nwidgets/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── circular_progress.rs
│   │   ├── corner.rs
│   │   ├── dropdown.rs
│   │   ├── element_ext.rs
│   │   ├── popover_menu.rs
│   │   ├── search_input.rs
│   │   ├── search_results.rs
│   │   └── toggle.rs
│   ├── services/            # System integration services
│   │   ├── audio.rs
│   │   ├── bluetooth.rs
│   │   ├── cef/             # Browser integration
│   │   ├── chat.rs
│   │   ├── clipboard.rs
│   │   ├── control_center.rs
│   │   ├── dbus.rs
│   │   ├── hyprland.rs
│   │   ├── launcher/
│   │   ├── lock_state.rs
│   │   ├── mpris.rs
│   │   ├── network/
│   │   ├── notifications.rs
│   │   ├── osd.rs
│   │   ├── pomodoro.rs
│   │   ├── system_monitor.rs
│   │   └── systray.rs
│   ├── widgets/             # Main UI widgets
│   │   ├── chat.rs
│   │   ├── control_center/
│   │   ├── launcher.rs
│   │   ├── notifications.rs
│   │   ├── osd.rs
│   │   └── panel/
│   ├── utils/               # Utility functions
│   │   ├── icon.rs
│   │   └── result_ext.rs
│   ├── theme.rs             # Theme configuration
│   └── main.rs              # Application entry point
├── assets/                  # Icons and resources
├── .ai/                     # Documentation
│   ├── docs/
│   │   └── gpui/            # GPUI framework documentation
│   ├── performance-guide.md
│   └── ...
└── Cargo.toml
```

## Technical Details

### GPUI Framework
This project uses a **custom fork of GPUI** with Wayland support:
```toml
gpui = { git = "https://github.com/Niahex/zed", features = ["wayland"] }
```

For GPUI documentation and examples, see `.ai/docs/gpui/`.

## Development

### Adding a New Service
1. Create service in `src/services/`
2. Implement event-driven updates with `tokio::Notify`
3. Add state comparison before emitting events
4. Initialize in `main.rs`

### Adding a New Widget
1. Create widget in `src/widgets/`
2. Implement `Render` trait
3. Subscribe to relevant services
4. Use deferred rendering for complex views

### Code Style
- Use `SharedString` for UI text
- Avoid clones in hot paths
- Prefer event-driven over polling
- Add state comparison before `cx.notify()`
- Use `cx.spawn()` for async work

## Troubleshooting

### High CPU Usage
Check `.ai/performance-guide.md` for debugging tips.

### Segfault
Usually caused by mutable iterator conflicts. Use immutable iterators in `render()`.

### CEF Not Loading
Ensure CEF subprocess is properly initialized before GPUI.

## License

GNU General Public License v3.0 - see LICENSE file for details.

## Credits

Built with:
- [GPUI](https://github.com/zed-industries/zed) - GPU-accelerated UI framework
- [Hyprland](https://hyprland.org/) - Wayland compositor
- [CEF](https://bitbucket.org/chromiumembedded/cef) - Chromium Embedded Framework

Inspired by:
- [Zed](https://zed.dev/) - Performance patterns
- [Waybar](https://github.com/Alexays/Waybar) - Panel design
- [Rofi](https://github.com/davatorium/rofi) - Launcher UX
