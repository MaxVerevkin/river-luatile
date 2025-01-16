#![warn(clippy::match_same_arms)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::unnecessary_wraps)]

#[macro_use]
extern crate log;

use mlua::prelude::*;
use river_layout_toolkit::{run, GeneratedLayout, Layout, Rectangle};

use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let lua = match LuaLayout::new() {
        Ok(lua) => lua,
        Err(err) => {
            error!("{err}");
            return ExitCode::FAILURE;
        }
    };

    match run(lua) {
        Ok(()) => unreachable!(),
        Err(err) => {
            error!("{err}");
            ExitCode::FAILURE
        }
    }
}

impl Layout for LuaLayout {
    type Error = LuaError;

    const NAMESPACE: &'static str = "luatile";

    fn user_cmd(&mut self, cmd: String, tags: Option<u32>, output: &str) -> LuaResult<()> {
        self.lua.globals().set("CMD_TAGS", tags)?;
        self.lua.globals().set("CMD_OUTPUT", output)?;

        self.lua.load(&cmd).exec()?;

        self.lua.globals().set("CMD_TAGS", LuaNil)?;
        self.lua.globals().set("CMD_OUTPUT", LuaNil)?;

        Ok(())
    }

    fn generate_layout(
        &mut self,
        view_count: u32,
        usable_width: u32,
        usable_height: u32,
        tags: u32,
        output: &str,
    ) -> LuaResult<GeneratedLayout> {
        let args = self.lua.create_table()?;
        args.set("tags", tags)?;
        args.set("count", view_count)?;
        args.set("width", usable_width)?;
        args.set("height", usable_height)?;
        args.set("output", output)?;

        let layout = self
            .lua
            .globals()
            .get::<LuaFunction>("handle_layout")?
            .call::<LuaTable>(args.clone())?;

        let metadata = self
            .lua
            .globals()
            .get::<Option<LuaFunction>>("handle_metadata")?
            .map(|f| f.call::<LuaTable>(args))
            .transpose()?;

        let name = metadata.as_ref().and_then(|m| m.get::<String>("name").ok());

        let mut generated_layout = GeneratedLayout {
            layout_name: name.unwrap_or_else(|| "luatile".to_string()),
            views: Vec::with_capacity(view_count as usize),
        };

        for view_geometry in layout.sequence_values::<LuaTable>() {
            let view_geometry = view_geometry?;
            let table_len = view_geometry.len()?;
            if table_len != 4 {
                return Err(LuaError::RuntimeError(format!(
                    "expected table of size 4, got {table_len}"
                )));
            }
            let mut it = view_geometry.sequence_values::<i32>();
            generated_layout.views.push(Rectangle {
                x: it.next().unwrap()?,
                y: it.next().unwrap()?,
                width: it
                    .next()
                    .unwrap()?
                    .try_into()
                    .map_err(|_| LuaError::RuntimeError("invalid view width".into()))?,
                height: it
                    .next()
                    .unwrap()?
                    .try_into()
                    .map_err(|_| LuaError::RuntimeError("invalid view height".into()))?,
            });
        }

        Ok(generated_layout)
    }
}

const DEFAULT_LAYOUT: &str = include_str!("../layout.lua");

struct LuaLayout {
    lua: Lua,
}

impl LuaLayout {
    fn new() -> LuaResult<Self> {
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
        lua.load(layout_str).exec()?;
        Ok(Self { lua })
    }
}

fn lua_layout_path() -> Option<PathBuf> {
    let mut path = config_dir()?;
    path.push("river-luatile");
    path.push("layout.lua");
    path.exists().then_some(path)
}

fn config_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(xdg.into());
    }

    #[allow(deprecated)] // Don't care about windows
    if let Some(mut path) = std::env::home_dir() {
        path.push(".config");
        return Some(path);
    }

    None
}
