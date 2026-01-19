# GPUI Introduction

## What is GPUI?

GPUI (GPU-accelerated UI) is a hybrid immediate and retained mode, GPU-accelerated UI framework for Rust, designed to support a wide variety of applications. Originally developed for the Zed code editor, GPUI provides high-performance rendering capabilities with a modern, declarative API.

## Key Features

- **GPU Acceleration**: Leverages Vulkan/OpenGL on NixOS
- **Hybrid Rendering**: Combines immediate and retained mode rendering for optimal performance
- **Wayland Native**: Native Wayland support with X11 fallback
- **Modern Rust**: Built with modern Rust patterns and zero-cost abstractions
- **Reactive State Management**: Entity-based state management with automatic updates
- **Flexible Layout**: CSS-like layout system with Flexbox support
- **Rich Text Rendering**: Advanced text rendering with font fallbacks and shaping
- **Animation Support**: Built-in animation system with easing functions
- **Accessibility**: Screen reader support and keyboard navigation

## Version Information

- **Current Version**: 0.2.2
- **License**: Apache-2.0
- **Repository**: https://github.com/zed-industries/zed
- **Homepage**: https://gpui.rs

## Design Philosophy

GPUI follows three main design principles:

1. **Performance First**: GPU acceleration and efficient rendering
2. **Developer Experience**: Modern Rust patterns and declarative API
3. **Cross-Platform**: Native feel on all supported platforms

## Use Cases

GPUI is ideal for:

- Code editors and IDEs
- Desktop applications requiring high performance
- Creative tools and media applications
- System utilities and developer tools
- Applications with complex UI requirements
- NixOS desktop applications and widgets
