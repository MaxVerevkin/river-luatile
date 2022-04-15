#[macro_use]
extern crate log;

use sctk::environment::{Environment, SimpleGlobal};
use sctk::output::{with_output_info, OutputHandler, OutputHandling, OutputInfo};
use sctk::reexports::client::Attached;
use sctk::reexports::{calloop, client};
use sctk::WaylandSource;

use client::protocol::wl_output::WlOutput;
use client::Display;
use client::Main;

use std::cell::RefCell;
use std::rc::Rc;

mod river_protocols;
use river_layout_manager_v3::RiverLayoutManagerV3 as RiverLayoutManager;
use river_layout_v3::RiverLayoutV3 as RiverLayout;
use river_protocols::river_layout_manager_v3;
use river_protocols::river_layout_v3;

mod lua_layout;
use lua_layout::LuaLayout;

fn main() {
    env_logger::init();
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
    let layouts = Layouts::new(&env);

    let output_handler = move |output: WlOutput, info: &OutputInfo| {
        if info.obsolete {
            layouts.remove(info.id);
            output.release();
        } else {
            layouts.push(output, info.id);
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

type LayoutObj = (u32, Main<RiverLayout>);

#[derive(Clone)]
struct Layouts {
    lua: LuaLayout,
    layout_manager: Attached<RiverLayoutManager>,
    layouts: Rc<RefCell<Vec<LayoutObj>>>,
}

impl Layouts {
    // TODO make configurable
    const NAMESPACE: &'static str = "luatile";

    fn new(env: &Environment<Env>) -> Self {
        Self {
            lua: LuaLayout::new(),
            layout_manager: env.require_global(),
            layouts: Default::default(),
        }
    }

    fn push(&self, output: WlOutput, output_id: u32) {
        info!("Creating RiverLayout object for output {}", output_id);
        let river_layout_obj = self
            .layout_manager
            .get_layout(&output, Self::NAMESPACE.to_owned());
        let layouts_handle = self.clone();
        river_layout_obj.quick_assign(move |layout, event, _| match event {
            river_protocols::river_layout_v3::Event::NamespaceInUse => {
                error!("Error: namespace '{}' is already in use", Self::NAMESPACE);
                layouts_handle.remove(output_id);
            }
            river_protocols::river_layout_v3::Event::LayoutDemand {
                view_count,
                usable_width,
                usable_height,
                tags,
                serial,
            } => match layouts_handle.lua.handle_layout(
                tags,
                view_count,
                usable_width,
                usable_height,
            ) {
                Ok(geometry) => {
                    for (x, y, w, h) in geometry {
                        layout.push_view_dimensions(x, y, w, h, serial);
                    }
                    layout.commit("luatile_layout".to_owned(), serial);
                }
                Err(e) => error!("Failed to handle layout demand: {e}"),
            },
            river_protocols::river_layout_v3::Event::UserCommand { command } => {
                if let Err(e) = layouts_handle.lua.handle_user_cmd(&command) {
                    error!("Failed to handle user command: {e}");
                }
            }
        });
        self.layouts
            .borrow_mut()
            .push((output_id, river_layout_obj));
    }

    fn remove(&self, output_id: u32) {
        info!("Destroying RiverLayout object for output {output_id}");
        self.layouts.borrow_mut().retain(|(id, layout)| {
            if *id == output_id {
                layout.destroy();
                false
            } else {
                true
            }
        })
    }
}
