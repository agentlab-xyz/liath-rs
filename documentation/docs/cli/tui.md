# Terminal User Interface (TUI)

Liath includes a full-featured terminal UI for interactive database exploration.

## Starting TUI

```bash
liath tui
# or simply
liath
```

## Interface Overview

```
┌─ Liath Database ─────────────────────────────────────────────────┐
│                                                                   │
│  Namespaces          │  Content                                   │
│  ─────────────────   │  ───────────────────────────────────────   │
│  > default           │  Key: greeting                             │
│    documents         │  Value: Hello, World!                      │
│    memories          │                                            │
│    agent:assistant   │                                            │
│                      │                                            │
├──────────────────────┴───────────────────────────────────────────┤
│  Query: _                                                         │
├──────────────────────────────────────────────────────────────────┤
│  [F1] Help  [F2] Namespaces  [F3] Search  [F5] Execute  [F10] Quit│
└──────────────────────────────────────────────────────────────────┘
```

## Navigation

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch between panels |
| `↑/↓` | Navigate lists |
| `Enter` | Select item |
| `Esc` | Cancel/Back |
| `F1` | Help |
| `F2` | Namespace browser |
| `F3` | Search |
| `F4` | Key-value browser |
| `F5` | Lua executor |
| `F10` / `q` | Quit |

### Mouse Support

- Click to select items
- Scroll to navigate lists
- Double-click to expand/collapse

## Panels

### Namespace Panel

Browse and manage namespaces:

- View all namespaces
- See namespace details (dimensions, metric, vector count)
- Create new namespaces
- Delete namespaces

**Keyboard:**

- `n` - Create new namespace
- `d` - Delete selected namespace
- `Enter` - Browse namespace contents

### Content Panel

View key-value pairs in selected namespace:

- Browse keys
- View values
- Edit values
- Delete entries

**Keyboard:**

- `a` - Add new key-value
- `e` - Edit selected value
- `d` - Delete selected key
- `/` - Search keys

### Query Panel

Execute Lua queries:

```
┌─ Query ──────────────────────────────────────────────────────────┐
│ return json.encode(semantic_search("docs", "machine learning", 5))│
│                                                                   │
│ Result:                                                           │
│ [{"id":"doc:1","content":"...","distance":0.123}]                 │
└──────────────────────────────────────────────────────────────────┘
```

**Keyboard:**

- `Ctrl+Enter` - Execute query
- `Ctrl+L` - Clear output
- `↑/↓` - Navigate history

### Search Panel

Semantic search interface:

```
┌─ Semantic Search ────────────────────────────────────────────────┐
│ Namespace: [documents    ▼]                                       │
│ Query:     [machine learning________________]                     │
│ Results:   [10    ]                                               │
│                                                                   │
│ ┌─ Results ─────────────────────────────────────────────────────┐│
│ │ 1. doc:1 - Introduction to neural... (0.123)                  ││
│ │ 2. doc:2 - Deep learning basics...   (0.234)                  ││
│ │ 3. doc:3 - ML model training...      (0.345)                  ││
│ └───────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

## Operations

### Creating a Namespace

1. Press `F2` to open namespace panel
2. Press `n` for new namespace
3. Enter details:
   - Name: `my_namespace`
   - Dimensions: `384`
   - Metric: `cosine`
   - Scalar: `f32`
4. Press `Enter` to create

### Storing Data

1. Select namespace
2. Press `F4` for key-value browser
3. Press `a` to add
4. Enter key and value
5. Press `Enter` to save

### Semantic Search

1. Press `F3` for search panel
2. Select namespace
3. Enter query
4. Set number of results
5. Press `Enter` to search
6. Browse results with `↑/↓`

### Executing Lua

1. Press `F5` for query panel
2. Enter Lua code
3. Press `Ctrl+Enter` to execute
4. View results below

## Themes

### Dark Theme (Default)

```bash
liath tui --theme dark
```

### Light Theme

```bash
liath tui --theme light
```

### Custom Colors

Set via environment:

```bash
export LIATH_TUI_BG="#1e1e2e"
export LIATH_TUI_FG="#cdd6f4"
export LIATH_TUI_ACCENT="#89b4fa"
liath tui
```

## Configuration

### Config File

`~/.config/liath/tui.toml`:

```toml
[theme]
name = "dark"

[layout]
sidebar_width = 30
show_line_numbers = true

[editor]
tab_size = 4
auto_indent = true

[history]
max_entries = 1000
save_on_exit = true
```

### Key Bindings

Custom key bindings in `~/.config/liath/keys.toml`:

```toml
[global]
quit = ["q", "F10"]
help = ["F1", "?"]

[navigation]
up = ["k", "Up"]
down = ["j", "Down"]
left = ["h", "Left"]
right = ["l", "Right"]

[editor]
execute = ["Ctrl+Enter"]
clear = ["Ctrl+l"]
```

## Advanced Features

### Split View

View multiple panels simultaneously:

- `Ctrl+\` - Vertical split
- `Ctrl+-` - Horizontal split
- `Ctrl+w` - Close split

### Bookmarks

Save frequently accessed items:

- `m` - Add bookmark
- `'` - Open bookmarks
- `d'` - Delete bookmark

### Export

Export data from TUI:

- `Ctrl+e` - Export selected as JSON
- `Ctrl+Shift+e` - Export namespace

## Troubleshooting

### Display Issues

If the TUI doesn't render correctly:

```bash
# Force 256 colors
export TERM=xterm-256color
liath tui

# Disable mouse
liath tui --no-mouse

# Simple mode (no borders)
liath tui --simple
```

### Performance

For large datasets:

```bash
# Limit displayed items
liath tui --max-items 1000

# Disable live preview
liath tui --no-preview
```

### Terminal Compatibility

Tested terminals:

- ✅ iTerm2
- ✅ Alacritty
- ✅ Kitty
- ✅ GNOME Terminal
- ✅ Windows Terminal
- ⚠️ macOS Terminal (limited colors)

## See Also

- [Commands](commands.md) - CLI commands
- [Interactive Mode](interactive.md) - REPL alternative
- [Quick Start](../getting-started/quick-start.md) - Getting started
