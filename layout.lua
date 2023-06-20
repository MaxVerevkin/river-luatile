-- You can define your global state here
local main_ratio = 0.65
local gaps = 10
local smart_gaps = false

-- The most important function - the actual layout generator
--
-- The argument is a table with:
--  * Focused tags (`args.tags`)
--  * Window count (`args.count`)
--  * Output width (`args.width`)
--  * Output height (`args.height`)
--  * Output name (`args.output`)
--
-- The return value must be a table with exactly `count` entries. Each entry is a table with four
-- numbers:
--  * X coordinate
--  * Y coordinate
--  * Window width
--  * Window height
--
-- This example is a simplified version of `rivertile`
function handle_layout(args)
	local retval = {}
	if args.count == 1 then
		if smart_gaps then
			table.insert(retval, { 0, 0, args.width, args.height })
		else
			table.insert(retval, { gaps, gaps, args.width - gaps * 2, args.height - gaps * 2 })
		end
	elseif args.count > 1 then
		local main_w = (args.width - gaps * 3) * main_ratio
		local side_w = (args.width - gaps * 3) - main_w
		local main_h = args.height - gaps * 2
		local side_h = (args.height - gaps) / (args.count - 1) - gaps
		table.insert(retval, {
			gaps,
			gaps,
			main_w,
			main_h,
		})
		for i = 0, (args.count - 2) do
			table.insert(retval, {
				main_w + gaps * 2,
				gaps + i * (side_h + gaps),
				side_w,
				side_h,
			})
		end
	end
	return retval
end

-- This optional function returns the metadata for the current layout.
-- Currently only `name` is supported, the name of the layout. It get's passed
-- the same `args` as handle_layout()
function handle_metadata(args)
	return { name = "rivertile-simple" }
end

-- IMPORTANT: User commands send via `riverctl send-layout-cmd` are treated as lua code.
-- Active tags are stored in `CMD_TAGS` global variable.
-- Output name is stored in `CMD_OUTPUT` global variable.

-- Here is an example of a function that can be mapped to some key
-- Run with `riverctl send-layout-cmd luatile "toggle_gaps()"`
local gaps_alt = 0
function toggle_gaps()
	local tmp = gaps
	gaps = gaps_alt
	gaps_alt = tmp
end
