# Spy-Code Neovim Plugin

Codebase intelligence for Neovim - index, query, and analyze your code with graph-based navigation.

## Features

- **Search Codebase**: Search for functions, classes, and constants
- **Find Callers**: See what functions call a selected symbol
- **Find Callees**: See what functions a selected symbol calls
- **Node Details**: View detailed information about code symbols
- **Telescope Integration**: Fuzzy find through search results
- **Floating Windows**: Display results in floating windows

## Installation

### Using vim-plug

```lua
Plug 'Psyborgs-git/spy-code.nvim'
```

### Using packer.nvim

```lua
use {
  'Psyborgs-git/spy-code.nvim',
  requires = { 'nvim-lua/plenary.nvim', 'nvim-telescope/telescope.nvim' }
}
```

### Using lazy.nvim

```lua
return {
  'Psyborgs-git/spy-code.nvim',
  dependencies = { 'nvim-lua/plenary.nvim', 'nvim-telescope/telescope.nvim' },
  config = function()
    require('spy-code').setup()
  end
}
```

## Prerequisites

1. Install spy-code globally:
   ```bash
   npm install -g spy-code
   # or
   pip install spy-code
   ```

2. Initialize spy-code in your project:
   ```bash
   cd your-project
   spy-code init
   spy-code index
   ```

## Usage

### Search Codebase

```vim
:SpyCodeSearch
```

Or with a query:
```vim
:SpyCodeSearch authenticate
```

### Find Callers

Place cursor on a symbol and run:
```vim
:SpyCodeFindCallers
```

### Find Callees

Place cursor on a symbol and run:
```vim
:SpyCodeFindCallees
```

### Show Node Details

Place cursor on a symbol and run:
```vim
:SpyCodeShowNode
```

### Index Codebase

```vim
:SpyCodeIndex
```

### Show Statistics

```vim
:SpyCodeStats
```

## Telescope Integration

```vim
:Telescope spy-code search
```

## Configuration

```lua
require('spy-code').setup({
  config_path = 'spy.config.json',
  auto_index = true,
  telescope_integration = true
})
```

## Key Mappings

Add to your `init.lua`:

```lua
local spycode = require('spy-code')

-- Search
vim.keymap.set('n', '<leader>ss', spycode.search, { desc = 'Spy-Code Search' })

-- Find callers
vim.keymap.set('n', '<leader>sc', spycode.find_callers, { desc = 'Spy-Code Find Callers' })

-- Find callees
vim.keymap.set('n', '<leader>sf', spycode.find_callees, { desc = 'Spy-Code Find Callees' })

-- Show node
vim.keymap.set('n', '<leader>sn', spycode.show_node, { desc = 'Spy-Code Show Node' })

-- Index
vim.keymap.set('n', '<leader>si', spycode.index, { desc = 'Spy-Code Index' })
```

## Requirements

- Neovim 0.7.0 or later
- spy-code CLI installed
- spy-code initialized in your workspace
- plenary.nvim (for async operations)
- telescope.nvim (optional, for fuzzy finding)

## License

MIT
