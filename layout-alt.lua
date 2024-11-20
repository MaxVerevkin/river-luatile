-- You can define your global state here
local main_ratio = 0.6
local gaps = 0
local border_width = 2
-- Track the last set border width (to avoid repeated changes)
local last_border_width = -1

-- The most important function - the actual layout generator
function handle_layout(args)
    local retval = {}

-- Smart borders: only draw borders if there is more than one client
--------------------------------------------------------------------
    local new_border_width = (args.count == 1) and 0 or border_width
    if new_border_width ~= last_border_width then
        os.execute("riverctl border-width " .. new_border_width) 
        last_border_width = new_border_width
    end

    -- Layout logic with adjustments to remove gaps at the top of the first and bottom of the last side windows
    if args.count == 1 then
        table.insert(retval, { gaps, gaps, args.width - gaps * 2, args.height - gaps * 2 })
    elseif args.count > 1 then
        local main_w = math.floor((args.width - gaps * 3) * main_ratio)
        local side_w = (args.width - gaps * 3) - main_w

        local main_h = math.floor(args.height - gaps * 2)
        local side_h = math.floor((args.height - gaps) / (args.count - 1) - gaps)
        local side_h_rem = (args.height - gaps) % (args.count - 1)  -- remainder for side height

        -- Main window
        table.insert(retval, {
            gaps,
            gaps,
	   main_w,
	   main_h,
        })

        -- Side windows
        for i = 0, (args.count - 2) do
            -- Adjust the vertical position for side windows
            local y_offset = gaps + i * (side_h + gaps)

            -- For the first side window, remove the gap at the top
            if i == 0 then
                y_offset = i * side_h  -- No extra gap at the top for the first window
            end

            -- For the last side window, adjust the height to avoid excess space
            local adjusted_height = side_h + (i == 0 and side_h_rem or 0)
            if i == args.count - 2 then
                adjusted_height = side_h + side_h_rem  -- Ensure the last side window doesn't leave space at the bottom
            end

            table.insert(retval, {
                main_w + gaps * 2,
                y_offset,
                side_w,
                adjusted_height,  -- Use adjusted height for side windows
            })
        end
    end
    return retval
end
