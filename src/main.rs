#![warn(clippy::match_same_arms)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::unnecessary_wraps)]

#[macro_use]
extern crate log;

use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::client::Connection,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};

mod river_layout;
use river_layout::{GeneratedLayout, RiverLayout, RiverLayoutHandler, RiverLayoutState};

mod lua_layout;
use lua_layout::LuaLayout;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_output;

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let mut state = State {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        river_layout_state: RiverLayoutState::new(&globals, &qh),

        lua: LuaLayout::new(),
        layouts: Vec::new(),
    };

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

pub struct State {
    registry_state: RegistryState,
    output_state: OutputState,
    river_layout_state: RiverLayoutState,

    lua: LuaLayout,
    layouts: Vec<(wl_output::WlOutput, RiverLayout)>,
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let new_layout = self
            .river_layout_state
            .new_layout(qh, &output, "luatile".into());
        self.layouts.push((output, new_layout));
    }

    fn update_output(
        &mut self,
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
        _: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        self.layouts.retain(|(o, _)| *o != output);
    }
}

impl RiverLayoutHandler for State {
    fn river_control_state(&mut self) -> &mut RiverLayoutState {
        &mut self.river_layout_state
    }

    fn namespace_in_use(&mut self) {
        panic!("Oh no... The namespace is in use!");
    }

    fn generate_layout(
        &mut self,
        view_count: u32,
        usable_width: u32,
        usable_height: u32,
        tags: u32,
    ) -> GeneratedLayout {
        self.lua
            .generate_layout(tags, view_count, usable_width, usable_height)
            .unwrap()
    }

    fn handle_user_cmd(&mut self, cmd: String) {
        self.lua.handle_user_cmd(&cmd).unwrap();
    }
}

delegate_output!(State);
delegate_registry!(State);
delegate_river_layout!(State);

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState,];
}
