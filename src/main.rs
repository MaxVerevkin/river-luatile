use sctk::environment::{Environment, SimpleGlobal};
use sctk::output::{with_output_info, OutputHandler, OutputHandling, OutputInfo};
use sctk::reexports::{calloop, client};
use sctk::WaylandSource;

use client::protocol::wl_output::WlOutput;
use client::Display;
use client::Main;

mod river_protocols;
use river_layout_manager_v3::RiverLayoutManagerV3 as RiverLayoutManager;
use river_layout_v3::RiverLayoutV3 as RiverLayout;
use river_protocols::river_layout_manager_v3;
use river_protocols::river_layout_v3;

mod lua_layout;
use lua_layout::LuaLayout;

fn main() {
    let lua_layout = LuaLayout::new();

    let display = Display::connect_to_env().unwrap();
    let mut queue = display.create_event_queue();

    let env = Environment::new(
        &display.attach(queue.token()),
        &mut queue,
        Env {
            layout_manager: SimpleGlobal::new(),
            outputs: OutputHandler::new(),
        },
    )
    .unwrap();
    let layout_manager = env.require_global::<RiverLayoutManager>();
    let mut layouts = Vec::<(u32, Main<RiverLayout>)>::new();

    let mut output_handler = move |output: WlOutput, info: &OutputInfo| {
        if info.obsolete {
            layouts.retain(|(id, layout)| {
                if *id == info.id {
                    layout.destroy();
                    false
                } else {
                    true
                }
            });
            output.release();
        } else {
            let namespace = "luatile".to_owned();
            let river_layout = layout_manager.get_layout(&output, namespace.clone());
            let lua_layout = lua_layout.clone();
            river_layout.quick_assign(move |layout, event, _| match event {
                river_protocols::river_layout_v3::Event::NamespaceInUse => {
                    eprintln!("Error: namespace '{namespace}' is already in use");
                }
                river_protocols::river_layout_v3::Event::LayoutDemand {
                    view_count,
                    usable_width,
                    usable_height,
                    tags,
                    serial,
                } => {
                    for (x, y, w, h) in
                        lua_layout.handle_layout(tags, view_count, usable_width, usable_height)
                    {
                        layout.push_view_dimensions(x, y, w, h, serial);
                    }
                    layout.commit("luatile_layout".to_owned(), serial);
                }
                river_protocols::river_layout_v3::Event::UserCommand { command } => {
                    lua_layout.handle_user_cmd(command);
                }
            });
            layouts.push((info.id, river_layout));
        }
    };
    for output in env.get_all_outputs() {
        if let Some(info) = with_output_info(&output, Clone::clone) {
            output_handler(output, &info);
        }
    }
    let _listner_handle =
        env.listen_for_outputs(move |output, info, _| output_handler(output, info));

    let mut event_loop = calloop::EventLoop::<()>::try_new().unwrap();
    WaylandSource::new(queue)
        .quick_insert(event_loop.handle())
        .unwrap();

    loop {
        display.flush().unwrap();
        if let Err(e) = event_loop.dispatch(None, &mut ()) {
            panic!("calloop error: {e}");
        }
    }
}

#[derive(Debug)]
struct Env {
    layout_manager: SimpleGlobal<RiverLayoutManager>,
    outputs: OutputHandler,
}

sctk::environment!(Env,
    singles = [
        RiverLayoutManager => layout_manager,
    ],
    multis = [
        WlOutput => outputs,
    ]
);

impl OutputHandling for Env {
    fn listen<F: FnMut(WlOutput, &OutputInfo, client::DispatchData) + 'static>(
        &mut self,
        f: F,
    ) -> sctk::output::OutputStatusListener {
        self.outputs.listen(f)
    }
}
