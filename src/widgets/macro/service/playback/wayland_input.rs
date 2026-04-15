use crate::widgets::r#macro::types::*;
use anyhow::{Context, Result};
use wayland_client::{
    protocol::{wl_registry, wl_seat},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
    zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
};

struct WaylandState {
    keyboard_manager: Option<ZwpVirtualKeyboardManagerV1>,
    pointer_manager: Option<ZwlrVirtualPointerManagerV1>,
    seat: Option<wl_seat::WlSeat>,
    virtual_keyboard: Option<ZwpVirtualKeyboardV1>,
    virtual_pointer: Option<ZwlrVirtualPointerV1>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &interface[..] {
                "zwp_virtual_keyboard_manager_v1" => {
                    state.keyboard_manager = Some(
                        registry.bind::<ZwpVirtualKeyboardManagerV1, _, _>(name, version, qh, ()),
                    );
                }
                "zwlr_virtual_pointer_manager_v1" => {
                    state.pointer_manager = Some(
                        registry.bind::<ZwlrVirtualPointerManagerV1, _, _>(name, version, qh, ()),
                    );
                }
                "wl_seat" => {
                    state.seat =
                        Some(registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ()));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ZwpVirtualKeyboardManagerV1,
        _: <ZwpVirtualKeyboardManagerV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ZwpVirtualKeyboardV1,
        _: <ZwpVirtualKeyboardV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrVirtualPointerManagerV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ZwlrVirtualPointerManagerV1,
        _: <ZwlrVirtualPointerManagerV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrVirtualPointerV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ZwlrVirtualPointerV1,
        _: <ZwlrVirtualPointerV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

pub struct WaylandInput {
    connection: Connection,
    state: WaylandState,
}

impl WaylandInput {
    pub fn new() -> Result<Self> {
        let connection =
            Connection::connect_to_env().context("Failed to connect to Wayland display")?;

        let display = connection.display();
        let mut event_queue = connection.new_event_queue();
        let qh = event_queue.handle();

        let mut state = WaylandState {
            keyboard_manager: None,
            pointer_manager: None,
            seat: None,
            virtual_keyboard: None,
            virtual_pointer: None,
        };

        let _registry = display.get_registry(&qh, ());

        event_queue.roundtrip(&mut state)?;

        if state.keyboard_manager.is_none() {
            anyhow::bail!("Compositor does not support zwp_virtual_keyboard_v1 protocol");
        }

        if state.pointer_manager.is_none() {
            anyhow::bail!("Compositor does not support zwlr_virtual_pointer_v1 protocol");
        }

        let seat = state.seat.as_ref().context("No wl_seat available")?;

        state.virtual_keyboard = Some(
            state
                .keyboard_manager
                .as_ref()
                .unwrap()
                .create_virtual_keyboard(seat, &qh, ()),
        );

        state.virtual_pointer = Some(
            state
                .pointer_manager
                .as_ref()
                .unwrap()
                .create_virtual_pointer(Some(seat), &qh, ()),
        );

        event_queue.roundtrip(&mut state)?;

        Ok(Self { connection, state })
    }

    pub fn send_key(&mut self, keycode: u32, pressed: bool) -> Result<()> {
        let keyboard = self
            .state
            .virtual_keyboard
            .as_ref()
            .context("Virtual keyboard not initialized")?;

        let state = if pressed { 1 } else { 0 };

        keyboard.key(0, keycode, state);

        let mut event_queue = self.connection.new_event_queue();
        event_queue.roundtrip(&mut self.state)?;

        Ok(())
    }

    pub fn move_pointer(&mut self, x: i32, y: i32) -> Result<()> {
        let pointer = self
            .state
            .virtual_pointer
            .as_ref()
            .context("Virtual pointer not initialized")?;

        pointer.motion_absolute(0, x as u32, y as u32, u32::MAX, u32::MAX);
        pointer.frame();

        let mut event_queue = self.connection.new_event_queue();
        event_queue.roundtrip(&mut self.state)?;

        Ok(())
    }

    pub fn click_button(&mut self, button: MacroMouseButton) -> Result<()> {
        use wayland_client::protocol::wl_pointer::ButtonState;

        let button_code = match button {
            MacroMouseButton::Left => 0x110,
            MacroMouseButton::Right => 0x111,
            MacroMouseButton::Middle => 0x112,
        };

        {
            let pointer = self
                .state
                .virtual_pointer
                .as_ref()
                .context("Virtual pointer not initialized")?;

            pointer.button(0, button_code, ButtonState::Pressed);
            pointer.frame();
        }

        {
            let mut event_queue = self.connection.new_event_queue();
            event_queue.roundtrip(&mut self.state)?;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));

        {
            let pointer = self
                .state
                .virtual_pointer
                .as_ref()
                .context("Virtual pointer not initialized")?;

            pointer.button(0, button_code, ButtonState::Released);
            pointer.frame();
        }

        {
            let mut event_queue = self.connection.new_event_queue();
            event_queue.roundtrip(&mut self.state)?;
        }

        Ok(())
    }
}
