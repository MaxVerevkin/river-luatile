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
            None => {
                info!("layout.lua not found: using default layout");
                DEFAULT_LAYOUT
            }
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
    ) -> LuaResult<Vec<(i32, i32, u32, u32)>> {
        let args = self.lua.create_table()?;
        args.set("tags", tags)?;
        args.set("count", count)?;
        args.set("width", width)?;
        args.set("height", height)?;

        let layout = self
            .lua
            .globals()
            .get::<_, LuaFunction>("handle_layout")?
            .call::<_, LuaTable>(args)?;
        let generated_count = layout.len()?;
        if generated_count != count as i64 {
            return Err(LuaError::RuntimeError(format!(
                "expected table of size {count}, got {generated_count}"
            )));
        }

        let mut retval = Vec::with_capacity(count as usize);
        for view_geometry in layout.sequence_values::<LuaTable>() {
            let view_geometry = view_geometry?;
            let table_len = view_geometry.len()?;
            if table_len != 4 {
                return Err(LuaError::RuntimeError(format!(
                    "expected table of size 4, got {table_len}"
                )));
            }
            let mut it = view_geometry.sequence_values::<i32>();
            retval.push((
                it.next().unwrap()?,
                it.next().unwrap()?,
                it.next()
                    .unwrap()?
                    .try_into()
                    .map_err(|_| LuaError::RuntimeError("invalid view width".into()))?,
                it.next()
                    .unwrap()?
                    .try_into()
                    .map_err(|_| LuaError::RuntimeError("invalid view height".into()))?,
            ));
        }
        Ok(retval)
    }

    pub fn handle_user_cmd(&self, cmd: &str) -> LuaResult<()> {
        self.lua.load(cmd).exec()
    }
}

fn lua_layout_path() -> Option<PathBuf> {
    let mut path = config_dir()?;
    path.push("river-luatile");
    path.push("layout.lua");
    path.exists().then(|| path)
}
