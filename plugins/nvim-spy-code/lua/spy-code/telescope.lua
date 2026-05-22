local M = {}

local function register_finder()
  local pickers = require('telescope.pickers')
  local finders = require('telescope.finders')
  local conf = require('telescope.config').values
  local actions = require('telescope.actions')
  local action_state = require('telescope.actions.state')
  
  M.search = function(opts)
    opts = opts or {}
    
    local query = opts.query or ''
    
    pickers.new(opts, {
      prompt_title = 'Spy-Code Search',
      finder = finders.new_dynamic({
        fn = function(prompt)
          if not prompt or prompt == '' then
            return {}
          end
          
          local cmd = 'spy-code search ' .. vim.fn.shellescape(prompt)
          local output = vim.fn.system(cmd)
          
          if vim.v.shell_error ~= 0 then
            return {}
          end
          
          -- Parse output into results
          local results = {}
          for line in output:gmatch('[^\r\n]+') do
            table.insert(results, {
              value = line,
              display = line,
              ordinal = line
            })
          end
          
          return results
        end,
      }),
      sorter = conf.generic_sorter(opts),
      attach_mappings = function(prompt_bufnr, map)
        actions.select_default:replace(function()
          local selection = action_state.get_selected_entry()
          actions.close(prompt_bufnr)
          
          -- Open the file at the location
          if selection and selection.value then
            -- Parse the line to extract file path and line number
            -- This is simplified - in production, parse the actual output
            vim.notify('Selected: ' .. selection.value, vim.log.levels.INFO)
          end
        end)
        return true
      end,
    }):find()
  end
end

function M.register()
  local has_telescope, telescope = pcall(require, 'telescope')
  if not has_telescope then
    vim.notify('telescope.nvim not found', vim.log.levels.WARN)
    return
  end
  
  register_finder()
  
  telescope.register_extension({
    exports = {
      spy_code = M.search
    }
  })
end

return M
