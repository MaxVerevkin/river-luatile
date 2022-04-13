-- You can define your global state here
local main_ratio = 0.65
local gaps = 10
local smart_gaps = false

-- The most important function - the actual layout generator
--
-- The argument is a table with:
--  * Focused tags
--  * Window count
--  * Output width
--  * Output height
--
-- The return value must be a table with exactly `count` entries. Each entry is a table with four
-- numbers:
--  * X coordinate
--  * Y coordinate
--  * Window width
--  * Window height
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

-- Handle `riverctl send-layout-cmd` events (optional)
function handle_user_cmd(cmd)
	if cmd == "main_ratio++" then
		main_ratio = math.min(0.9, main_ratio + 0.005)
	elseif cmd == "main_ratio--" then
		main_ratio = math.max(0.1, main_ratio - 0.005)
	elseif cmd == "gaps++" then
		gaps = gaps + 2
	elseif cmd == "gaps--" then
		gaps = math.max(0, gaps - 2)
	elseif cmd == "smart_gaps on" then
		smart_gaps = true
	elseif cmd == "smart_gaps off" then
		smart_gaps = false
	end
end
