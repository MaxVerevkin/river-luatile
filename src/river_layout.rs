pub mod protocol {
    #![allow(non_upper_case_globals)]
    #![allow(clippy::all)]

    use smithay_client_toolkit::reexports::client as wayland_client;
    use smithay_client_toolkit::reexports::client::protocol::*;

    pub mod __interfaces {
        use smithay_client_toolkit::reexports::client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("protocols/river-layout-v3.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("protocols/river-layout-v3.xml");
}

use std::sync::{Arc, Weak};

use smithay_client_toolkit::globals::GlobalData;
use wayland_client::globals::GlobalList;
use wayland_client::protocol::wl_output;
use wayland_client::{Connection, Dispatch, QueueHandle};

use protocol::river_layout_manager_v3;
use protocol::river_layout_v3;

#[derive(Debug)]
pub struct RiverLayoutState {
    manager: river_layout_manager_v3::RiverLayoutManagerV3,
    layouts: Vec<Weak<RiverLayoutInner>>,
}

impl RiverLayoutState {
    pub fn new<D>(global_list: &GlobalList, qh: &QueueHandle<D>) -> Self
    where
        D: Dispatch<river_layout_manager_v3::RiverLayoutManagerV3, GlobalData> + 'static,
    {
        RiverLayoutState {
            manager: global_list.bind(qh, 1..=1, GlobalData).unwrap(),
            layouts: Vec::new(),
        }
    }

    pub fn new_layout<D>(
        &mut self,
        qh: &QueueHandle<D>,
        output: &wl_output::WlOutput,
        namespace: String,
    ) -> RiverLayout
    where
        D: Dispatch<river_layout_manager_v3::RiverLayoutManagerV3, GlobalData>
            + Dispatch<river_layout_v3::RiverLayoutV3, RiverLayoutEventData>
            + 'static,
    {
        let layout = self
            .manager
            .get_layout(output, namespace, qh, RiverLayoutEventData {});
        let layout = RiverLayout(Arc::new(RiverLayoutInner { layout }));
        self.layouts.push(Arc::downgrade(&layout.0));
        layout
    }

    pub fn get_output_layout(
        &self,
        layout: &river_layout_v3::RiverLayoutV3,
    ) -> Option<RiverLayout> {
        self.layouts
            .iter()
            .filter_map(Weak::upgrade)
            .find(|inner| &inner.layout == layout)
            .map(RiverLayout)
    }
}

#[derive(Debug)]
pub struct GeneratedLayout {
    pub layout_name: String,
    pub dimentions: Vec<(i32, i32, u32, u32)>,
}

pub trait RiverLayoutHandler: Sized {
    fn river_control_state(&mut self) -> &mut RiverLayoutState;

    fn namespace_in_use(&mut self);

    fn generate_layout(
        &mut self,
        view_count: u32,
        usable_width: u32,
        usable_height: u32,
        tags: u32,
    ) -> GeneratedLayout;

    fn handle_user_cmd(&mut self, cmd: String);
}

#[derive(Debug)]
pub struct RiverLayoutEventData {
    // This is empty right now, but may be populated in the future.
}

#[macro_export]
macro_rules! delegate_river_layout {
    ($ty: ty) => {
        ::smithay_client_toolkit::reexports::client::delegate_dispatch!($ty: [
            $crate::river_layout::protocol::river_layout_manager_v3::RiverLayoutManagerV3: ::smithay_client_toolkit::globals::GlobalData
        ] => $crate::river_layout::RiverLayoutState);
        ::smithay_client_toolkit::reexports::client::delegate_dispatch!($ty: [
            $crate::river_layout::protocol::river_layout_v3::RiverLayoutV3: $crate::river_layout::RiverLayoutEventData
        ] => $crate::river_layout::RiverLayoutState);
    };
}

#[derive(Debug)]
pub struct RiverLayout(Arc<RiverLayoutInner>);

#[derive(Debug)]
struct RiverLayoutInner {
    layout: river_layout_v3::RiverLayoutV3,
}

impl Drop for RiverLayoutInner {
    fn drop(&mut self) {
        self.layout.destroy();
    }
}

impl<D> Dispatch<river_layout_manager_v3::RiverLayoutManagerV3, GlobalData, D> for RiverLayoutState
where
    D: Dispatch<river_layout_manager_v3::RiverLayoutManagerV3, GlobalData>
        + RiverLayoutHandler
        + 'static,
{
    fn event(
        _: &mut D,
        _: &river_layout_manager_v3::RiverLayoutManagerV3,
        _: river_layout_manager_v3::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<D>,
    ) {
        unreachable!("river_layout_manager_v3 has no events")
    }
}

impl<D> Dispatch<river_layout_v3::RiverLayoutV3, RiverLayoutEventData, D> for RiverLayoutState
where
    D: Dispatch<river_layout_v3::RiverLayoutV3, RiverLayoutEventData>
        + RiverLayoutHandler
        + 'static,
{
    fn event(
        data: &mut D,
        layout: &river_layout_v3::RiverLayoutV3,
        event: river_layout_v3::Event,
        _: &RiverLayoutEventData,
        _: &Connection,
        _: &QueueHandle<D>,
    ) {
        use river_layout_v3::Event;

        // Remove any layouts that have been dropped
        data.river_control_state()
            .layouts
            .retain(|l| l.upgrade().is_some());

        if let Some(layout) = data.river_control_state().get_output_layout(layout) {
            match event {
                Event::NamespaceInUse => {
                    data.namespace_in_use();
                }
                Event::LayoutDemand {
                    view_count,
                    usable_width,
                    usable_height,
                    tags,
                    serial,
                } => {
                    let generated_layout =
                        data.generate_layout(view_count, usable_width, usable_height, tags);
                    assert_eq!(generated_layout.dimentions.len(), view_count as usize);
                    for (x, y, w, h) in generated_layout.dimentions {
                        layout.0.layout.push_view_dimensions(x, y, w, h, serial);
                    }
                    layout.0.layout.commit(generated_layout.layout_name, serial);
                }
                Event::UserCommand { command } => {
                    data.handle_user_cmd(command);
                }
            }
        }
    }
}
