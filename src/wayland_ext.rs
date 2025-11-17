use gpui::Window;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Extension trait to set Wayland input region for a window
pub trait WaylandWindowExt {
    /// Sets an empty input region for the window, making it click-through
    /// This only works on Wayland platforms
    fn set_input_passthrough(&mut self);
}

impl WaylandWindowExt for Window {
    fn set_input_passthrough(&mut self) {
        println!("[WAYLAND_EXT] set_input_passthrough called");

        // Safety: This is platform-specific code for Wayland
        #[cfg(target_os = "linux")]
        {
            use std::ffi::c_void;

            println!("[WAYLAND_EXT] Getting window handle...");

            // Get the raw window handle - Window implements HasWindowHandle
            if let Ok(handle) = self.window_handle() {
                println!("[WAYLAND_EXT] Got window handle, checking if Wayland...");
                if let RawWindowHandle::Wayland(wayland_handle) = handle.as_raw() {
                    // Get the wl_surface pointer
                    let surface_ptr = wayland_handle.surface.as_ptr();

                    println!("[WAYLAND_EXT] Wayland surface pointer: {:?}", surface_ptr);
                    println!("[WAYLAND_EXT] Setting input passthrough for window");

                    unsafe {
                        // Wayland protocol constant for wl_surface::set_input_region
                        const WL_SURFACE_SET_INPUT_REGION: u32 = 5;

                        // Wayland wire protocol format for this call:
                        // - opcode: 5 (set_input_region)
                        // - argument: object ID (nullable wl_region)
                        //
                        // We're calling with NULL (0) to make the entire surface click-through

                        // Function signature from wayland-scanner C code:
                        // void wl_proxy_marshal(struct wl_proxy *proxy, uint32_t opcode, ...);
                        //
                        // The wayland-client library is dynamically linked by wayland_client crate
                        // We can use dlsym to get the function pointer

                        #[cfg(target_os = "linux")]
                        {
                            type WlProxyMarshal = unsafe extern "C" fn(*mut c_void, u32, ...);

                            // Load libwayland-client.so dynamically
                            let lib = libc::dlopen(
                                b"libwayland-client.so.0\0".as_ptr() as *const i8,
                                libc::RTLD_LAZY | libc::RTLD_GLOBAL
                            );

                            if lib.is_null() {
                                eprintln!("[WAYLAND_EXT] Failed to load libwayland-client.so.0");
                                return;
                            }

                            let wl_proxy_marshal = libc::dlsym(
                                lib,
                                b"wl_proxy_marshal\0".as_ptr() as *const i8
                            );

                            if wl_proxy_marshal.is_null() {
                                eprintln!("[WAYLAND_EXT] Failed to find wl_proxy_marshal");
                                libc::dlclose(lib);
                                return;
                            }

                            let marshal_fn: WlProxyMarshal = std::mem::transmute(wl_proxy_marshal);

                            // Call wl_surface_set_input_region(surface, NULL)
                            marshal_fn(
                                surface_ptr as *mut c_void,
                                WL_SURFACE_SET_INPUT_REGION,
                                std::ptr::null_mut::<c_void>()
                            );

                            println!("[WAYLAND_EXT] Called set_input_region(NULL)");

                            // IMPORTANT: We must call wl_surface::commit() for changes to take effect!
                            const WL_SURFACE_COMMIT: u32 = 6;
                            marshal_fn(
                                surface_ptr as *mut c_void,
                                WL_SURFACE_COMMIT
                            );

                            println!("[WAYLAND_EXT] Called commit()");

                            libc::dlclose(lib);

                            println!("[WAYLAND_EXT] Input region set to NULL (passthrough enabled)");
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("[WAYLAND_EXT] set_input_passthrough is only supported on Wayland");
        }
    }
}
