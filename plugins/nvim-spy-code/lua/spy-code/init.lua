local M = {}

local config = {
  config_path = 'spy.config.json',
  auto_index = true,
  telescope_integration = true
}

function M.setup(user_config)
  config = vim.tbl_extend('force', config, user_config or {})
  
  -- Load telescope extension if available
  if config.telescope_integration and pcall(require, 'telescope') then
    require('spy-code.telescope').register()
  end
  
  -- Register commands
  M.register_commands()
  
  -- Auto-index if enabled
  if config.auto_index then
    vim.api.nvim_create_autocmd('BufWritePost', {
      callback = function()
        M.index()
      end,
    })
  end
end

function M.register_commands()
  vim.api.nvim_create_user_command('SpyCodeSearch', function(opts)
    M.search(opts.args)
  end, { nargs = '?', complete = M.search_complete })
  
  vim.api.nvim_create_user_command('SpyCodeFindCallers', function()
    M.find_callers()
  end, {})
  
  vim.api.nvim_create_user_command('SpyCodeFindCallees', function()
    M.find_callees()
  end, {})
  
  vim.api.nvim_create_user_command('SpyCodeShowNode', function()
    M.show_node()
  end, {})
  
  vim.api.nvim_create_user_command('SpyCodeIndex', function()
    M.index()
  end, {})
  
  vim.api.nvim_create_user_command('SpyCodeStats', function()
    M.stats()
  end, {})
end

function M.search(query)
  if not query or query == '' then
    query = vim.fn.input('Search query: ')
  end
  
  if not query or query == '' then
    return
  end
  
  local cmd = 'spy-code search ' .. vim.fn.shellescape(query)
  local output = vim.fn.system(cmd)
  
  if vim.v.shell_error ~= 0 then
    vim.notify('Spy-Code search failed: ' .. output, vim.log.levels.ERROR)
    return
  end
  
  -- Parse output and show results
  M.show_search_results(output)
end

function M.find_callers()
  local node_id = M.get_node_id_under_cursor()
  if not node_id then
    vim.notify('Could not determine node ID', vim.log.levels.WARN)
    return
  end
  
  local cmd = 'spy-code callers ' .. vim.fn.shellescape(node_id)
  local output = vim.fn.system(cmd)
  
  if vim.v.shell_error ~= 0 then
    vim.notify('Spy-Code find callers failed: ' .. output, vim.log.levels.ERROR)
    return
  end
  
  M.show_callers(output)
end

function M.find_callees()
  local node_id = M.get_node_id_under_cursor()
  if not node_id then
    vim.notify('Could not determine node ID', vim.log.levels.WARN)
    return
  end
  
  local cmd = 'spy-code callees ' .. vim.fn.shellescape(node_id)
  local output = vim.fn.system(cmd)
  
  if vim.v.shell_error ~= 0 then
    vim.notify('Spy-Code find callees failed: ' .. output, vim.log.levels.ERROR)
    return
  end
  
  M.show_callees(output)
end

function M.show_node()
  local node_id = M.get_node_id_under_cursor()
  if not node_id then
    vim.notify('Could not determine node ID', vim.log.levels.WARN)
    return
  end
  
  local cmd = 'spy-code get ' .. vim.fn.shellescape(node_id)
  local output = vim.fn.system(cmd)
  
  if vim.v.shell_error ~= 0 then
    vim.notify('Spy-Code get node failed: ' .. output, vim.log.levels.ERROR)
    return
  end
  
  M.show_node_details(output)
end

function M.index()
  local cmd = 'spy-code index'
  local output = vim.fn.system(cmd)
  
  if vim.v.shell_error ~= 0 then
    vim.notify('Spy-Code index failed: ' .. output, vim.log.levels.ERROR)
    return
  end
  
  vim.notify('Codebase indexed successfully', vim.log.levels.INFO)
end

function M.stats()
  local cmd = 'spy-code stats'
  local output = vim.fn.system(cmd)
  
  if vim.v.shell_error ~= 0 then
    vim.notify('Spy-Code stats failed: ' .. output, vim.log.levels.ERROR)
    return
  end
  
  M.show_stats(output)
end

function M.get_node_id_under_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local filepath = vim.api.nvim_buf_get_name(bufnr)
  local cwd = vim.fn.getcwd()
  
  -- Get relative path
  local relative_path = filepath:gsub(cwd .. '/', '')
  -- Remove extension
  local no_ext = relative_path:gsub('%.[^.]+$', '')
  -- Convert to node ID format
  local node_id = no_ext:gsub('/', ':') .. ':_:'
  
  -- Get symbol under cursor
  local symbol = vim.fn.expand('<cword>')
  
  if symbol and symbol ~= '' then
    node_id = node_id .. symbol
  else
    return nil
  end
  
  return node_id
end

function M.show_search_results(output)
  local lines = vim.split(output, '\n')
  local buf = vim.api.nvim_create_buf(false, true)
  
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  
  local width = math.min(80, vim.o.columns - 10)
  local height = math.min(20, #lines)
  
  local win = vim.api.nvim_open_win(buf, true, {
    relative = 'editor',
    width = width,
    height = height,
    row = (vim.o.lines - height) / 2,
    col = (vim.o.columns - width) / 2,
    style = 'minimal',
    border = 'rounded'
  })
  
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)
  vim.api.nvim_win_set_option(win, 'cursorline', true)
  
  -- Close on Enter
  vim.keymap.set('n', '<CR>', function()
    vim.api.nvim_win_close(win, true)
  end, { buffer = buf })
  
  -- Close on Escape
  vim.keymap.set('n', '<Esc>', function()
    vim.api.nvim_win_close(win, true)
  end, { buffer = buf })
end

function M.show_callers(output)
  M.show_search_results(output)
end

function M.show_callees(output)
  M.show_search_results(output)
end

function M.show_node_details(output)
  M.show_search_results(output)
end

function M.show_stats(output)
  vim.notify(output, vim.log.levels.INFO)
end

function M.search_complete(arg_lead, cmd_line, cursor_pos)
  -- Provide completion for search (simplified)
  return {}
end

return M
