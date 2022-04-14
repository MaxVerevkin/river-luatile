use dirs_next::config_dir;
use mlua::prelude::*;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::rc::Rc;

const DEFAULT_LAYOUT: &str = include_str!("../layout.lua");

#[derive(Clone)]
pub struct LuaLayout {
    lua: Rc<Lua>,
}

impl LuaLayout {
    pub fn new() -> Self {
        let buf;
        let layout_str = match lua_layout_path() {
            Some(path) => {
                buf = read_to_string(path).unwrap();
                &buf
            }
            None => DEFAULT_LAYOUT,
        };
        let lua = Lua::new();
        lua.load(layout_str).exec().unwrap();
        Self { lua: Rc::new(lua) }
    }

    pub fn handle_layout(
        &self,
        tags: u32,
        count: u32,
        width: u32,
        height: u32,
    ) -> Vec<(i32, i32, u32, u32)> {
        // that's... 12 unwrap()s in one place...
        let args = self.lua.create_table().unwrap();
        args.set("tags", tags).unwrap();
        args.set("count", count).unwrap();
        args.set("width", width).unwrap();
        args.set("height", height).unwrap();
        let handler = self
            .lua
            .globals()
            .get::<_, LuaFunction>("handle_layout")
            .unwrap();
        let layouts = handler
            .call::<_, LuaTable>(args)
            .unwrap()
            .pairs::<u32, LuaTable>();
        let mut retval = Vec::new();
        for layout in layouts {
            let (_, layout) = layout.unwrap();
            let mut layout = layout.sequence_values();
            retval.push((
                layout.next().unwrap().unwrap(),
                layout.next().unwrap().unwrap(),
                layout.next().unwrap().unwrap() as u32,
                layout.next().unwrap().unwrap() as u32,
            ));
        }
        retval
    }

    pub fn handle_user_cmd(&self, cmd: String) {
        if let Err(error) = self.lua.load(&cmd).exec() {
            eprintln!("handle_user_cmd error: {error}");
        }
    }
}

fn lua_layout_path() -> Option<PathBuf> {
    let mut path = config_dir()?;
    path.push("river-luatile");
    path.push("layout.lua");
    path.exists().then(|| path)
}
