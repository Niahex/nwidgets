# NixOS Platform Support

GPUI native support for NixOS with Wayland and X11 backends.

## Installation

### NixOS Configuration

Add to your `configuration.nix`:

```nix
{ config, pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    # Wayland support
    wayland
    wayland-protocols
    libxkbcommon
    
    # Graphics
    vulkan-loader
    vulkan-validation-layers
    mesa
    
    # X11 fallback
    xorg.libX11
    xorg.libXrandr
    xorg.libXi
    
    # Development
    pkg-config
    cmake
  ];
  
  # Enable Wayland
  services.xserver.displayManager.gdm.wayland = true;
  
  # Graphics drivers
  hardware.opengl = {
    enable = true;
    driSupport = true;
    driSupport32Bit = true;
  };
}
```

### Development Shell

Create `shell.nix`:

```nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy
    
    # GPUI dependencies
    wayland
    wayland-protocols
    libxkbcommon
    vulkan-loader
    pkg-config
    
    # Optional X11 support
    xorg.libX11
    xorg.libXrandr
    xorg.libXi
  ];
  
  shellHook = ''
    export WAYLAND_DISPLAY=wayland-0
    export XDG_RUNTIME_DIR=/run/user/$(id -u)
  '';
}
```

### Flake Support

Create `flake.nix`:

```nix
{
  description = "GPUI Application";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            wayland
            libxkbcommon
            vulkan-loader
            pkg-config
          ];
        };
      });
}
```

## Graphics Configuration

### Vulkan Setup

```nix
# In configuration.nix
hardware.opengl = {
  enable = true;
  extraPackages = with pkgs; [
    vulkan-loader
    vulkan-validation-layers
  ];
};
```

### Environment Variables

```bash
# Wayland
export WAYLAND_DISPLAY=wayland-0
export XDG_RUNTIME_DIR=/run/user/$(id -u)

# Vulkan
export VK_LAYER_PATH=${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d
```

## Desktop Integration

### Application Package

```nix
{ pkgs, ... }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "my-gpui-app";
  version = "0.1.0";
  
  src = ./.;
  
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  
  nativeBuildInputs = with pkgs; [
    pkg-config
    wayland
  ];
  
  buildInputs = with pkgs; [
    wayland
    libxkbcommon
    vulkan-loader
  ];
  
  meta = {
    description = "My GPUI Application";
    license = pkgs.lib.licenses.mit;
  };
}
```

### Desktop Entry

```nix
# Add to your package
postInstall = ''
  mkdir -p $out/share/applications
  cat > $out/share/applications/my-app.desktop << EOF
[Desktop Entry]
Name=My GPUI App
Exec=$out/bin/my-app
Icon=my-app
Type=Application
Categories=Development;
EOF
'';
```

## Compositor Support

### Tested Compositors
- **Sway** - Recommended
- **Hyprland** - Full support
- **River** - Basic support
- **GNOME Shell** (Wayland)
- **KDE Plasma** (Wayland)

### Configuration Examples

#### Sway
```
# ~/.config/sway/config
exec my-gpui-app
for_window [app_id="my-app"] floating enable
```

#### Hyprland
```
# ~/.config/hypr/hyprland.conf
exec-once = my-gpui-app
windowrule = float, ^(my-app)$
```

## Troubleshooting

### Common Issues

#### Wayland Not Available
```bash
# Check Wayland session
echo $WAYLAND_DISPLAY

# Force X11 fallback
unset WAYLAND_DISPLAY
export DISPLAY=:0
```

#### Graphics Issues
```bash
# Check Vulkan
vulkaninfo

# Test OpenGL
glxinfo | grep "direct rendering"
```

#### Font Issues
```nix
# Add fonts to configuration.nix
fonts.packages = with pkgs; [
  dejavu_fonts
  liberation_ttf
  noto-fonts
];
```

### Performance Optimization

```nix
# Enable hardware acceleration
hardware.opengl = {
  enable = true;
  driSupport = true;
  extraPackages = with pkgs; [
    intel-media-driver  # Intel
    vaapiVdpau         # NVIDIA
    libvdpau-va-gl     # AMD
  ];
};
```

## Best Practices

1. **Use Flakes**: Better reproducibility
2. **Pin Dependencies**: Avoid version conflicts  
3. **Test Both Backends**: Wayland and X11
4. **Monitor Resources**: Memory and GPU usage
5. **Follow NixOS Patterns**: Use overlays for custom packages
