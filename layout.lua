-- You can define your global state here
local main_ratio = 0.4
local gaps = 10
local smart_gaps = false

-- The most important function - the actual layout generator
-- The argument is a table with:
--  * Tags
--  * Count
--  * Width
--  * Height
function handle_layout(args)
	local side_w = args.width * main_ratio
	local main_h = args.height - gaps * 2
	local main_w = args.width - gaps * 3 - side_w
	local retval = {}
	if args.count == 1 then
		if smart_gaps then
			table.insert(retval, { 0, 0, args.width, args.height })
		else
			table.insert(retval, { gaps, gaps, args.width - gaps * 2, args.height - gaps * 2 })
		end
	elseif args.count > 1 then
		local side_h = (args.height - gaps) / (args.count - 1) - gaps
		table.insert(retval, {
			gaps,
			gaps,
			args.width - gaps * 3 - side_w,
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
		main_ratio = math.min(0.9, main_ratio + 0.05)
	elseif cmd == "main_ratio--" then
		main_ratio = math.max(0.1, main_ratio - 0.05)
	elseif cmd == "smart_gaps on" then
		smart_gaps = true
	elseif cmd == "smart_gaps off" then
		smart_gaps = false
	end
end
