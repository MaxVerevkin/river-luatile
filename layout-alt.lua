-- You can define your global state here
local main_ratio = 0.6
local border_width = 2
-- Track the last set border width (to avoid repeated changes)
local last_border_width = -1
-- The most important function - the actual layout generator
function handle_layout(args)
    local retval = {}
    -- Smart borders: only draw borders if there is more than one client
    local new_border_width = (args.count == 1) and 0 or border_width
    if new_border_width ~= last_border_width then
        os.execute("riverctl border-width " .. new_border_width) 
        last_border_width = new_border_width
    end
    -- Layout logic with adjustments to make it pixel-perfect
    if args.count == 1 then
        table.insert(retval, { 0, 0, args.width, args.height })
    elseif args.count > 1 then
        local main_w = math.floor(args.width * main_ratio)
        local side_w = args.width - main_w
        local main_h = math.floor(args.height)
        local side_h = math.floor(args.height / (args.count - 1))
        local side_h_rem = args.height % (args.count - 1)
        table.insert(retval, { 0, 0, main_w, main_h })
        for i = 0, (args.count - 2) do
            table.insert(retval, { main_w, i * side_h, side_w, side_h + side_h_rem })
        end
    end
    return retval
end
