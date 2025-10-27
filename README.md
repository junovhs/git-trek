# 🚀 git-trek

```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ██████╗ ██╗████████╗   ████████╗██████╗ ███████╗██╗  ██╗  ║
║  ██╔════╝ ██║╚══██╔══╝   ╚══██╔══╝██╔══██╗██╔════╝██║ ██╔╝  ║
║  ██║  ███╗██║   ██║         ██║   ██████╔╝█████╗  █████╔╝   ║
║  ██║   ██║██║   ██║         ██║   ██╔══██╗██╔══╝  ██╔═██╗   ║
║  ╚██████╔╝██║   ██║         ██║   ██║  ██║███████╗██║  ██╗  ║
║   ╚═════╝ ╚═╝   ╚═╝         ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝  ║
║                                                               ║
║            Navigate Git History Like It's 1989!              ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
```

> *"Where we're going, we don't need `git log --graph`"*

## 🎮 What is this?

Ever wanted to scrub through your git history like you're editing video in the 90s? **git-trek** lets you navigate commits with the elegance of Oregon Trail and the power of a time machine.

## 🏁 Quick Start

```bash
# Install from crates.io (once published)
cargo install git-trek

# OR build from source
git clone https://github.com/yourusername/git-trek
cd git-trek
cargo install --path .

# Start trekking!
git-trek
```

## 🕹️ Controls

Once you start git-trek, you're in the captain's seat:

| Key | Action |
|-----|--------|
| `↑` `W` | Navigate to previous commit |
| `↓` `S` | Navigate to next commit |
| `A-J` | Jump directly to labeled commit |
| `R` | Restore (apply current commit to your branch) |
| `Q` | Quit (return to original state) |
| `X` | Exit (same as quit) |

## 🎯 Features

- **Visual Timeline**: See where you are (◉), where you started (◎), and everywhere else (○)
- **Letter Navigation**: Each visible commit gets a letter A-J for instant jumping
- **Safe Exploration**: Uses a temporary branch so your work is always safe
- **Retro Aesthetics**: Full color TUI that looks like it escaped from 1989
- **Zero Dependencies**: Single binary, works anywhere Rust works

## 📖 How It Works

1. **Start Trek**: Creates a temporary branch at your current HEAD
2. **Navigate**: Move through history without affecting your actual branch
3. **Restore or Quit**: Either apply a found commit or return home safely

```bash
# Start exploring from current HEAD
git-trek

# Navigate with arrow keys or WASD
# See a commit you like? Press R to restore to it
# Changed your mind? Press Q to quit without changes
```

## 🎨 The Interface

```
╔══════════════════════════════════════════════════════════════╗
║ 🚀 GIT TREK - STARDATE 2024                                 ║
╚══════════════════════════════════════════════════════════════╝
╔═══ TEMPORAL FLUX NAVIGATOR ═══╗
║  ◉ [A] Fix navigation bug      ║
║  │                              ║
║  ○ [B] Add color support        ║
║  │                              ║
║  ○ [C] Initial commit           ║
║  │                              ║
║  ◎ [D] Where you started        ║
╚═════════════════════════════════╝
╔═══ SCAN RESULTS ═══╗
║ Hash: abc123  Author: Captain   ║
║ Time: 2 hours ago  Changes: +42 ║
╚═════════════════════════════════╝
↑↓/WS: Navigate | A-J: Jump | R: Restore | Q: Quit
```

## 🚨 Requirements

- Git repository with commits to explore
- Clean working tree (commit or stash changes first)
- Terminal with color support (most modern terminals)

## 🛠️ Development

```bash
# Clone the repo
git clone https://github.com/yourusername/git-trek
cd git-trek

# Run in development
cargo run

# Run tests
cargo test

# Build optimized binary
cargo build --release

# Install locally
cargo install --path .
```

## 🎪 Why "git-trek"?

Because navigating git history should feel like an adventure, not a chore. This tool brings the joy of visual timeline scrubbing to the command line, wrapped in the cozy aesthetics of retro computing.

## 📜 License

MIT - Go forth and trek!

## 🙏 Inspired By

- The timeline scrubbing of video editors
- The simplicity of `git-nav` (the bash predecessor)
- The beauty of lazygit (but way simpler)
- Oregon Trail (for the vibes)
- Star Trek (for the name)

---

*Made with coffee and rust by developers who think git should be more fun*
