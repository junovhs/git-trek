```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ██████╗ ██╗████████╗   ████████╗██████╗ ███████╗██╗  ██╗    ║
║  ██╔════╝ ██║╚══██╔══╝   ╚══██╔══╝██╔══██╗██╔════╝██║ ██╔╝    ║
║  ██║  ███╗██║   ██║         ██║   ██████╔╝█████╗  █████╔╝     ║
║  ██║   ██║██║   ██║         ██║   ██╔══██╗██╔══╝  ██╔═██╗     ║
║  ╚██████╔╝██║   ██║         ██║   ██║  ██║███████╗██║  ██╗    ║
║   ╚═════╝ ╚═╝   ╚═╝         ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝    ║
║                                                               ║
║         Navigate Git History Like SNES Save Files!            ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
```

> *"Captain's Log, Stardate 2025: Where we're going, we don't need `git log --graph`."*

<p align="center">
  <img src="assets/demo.gif" alt="git-trek demo" width="100%" />
</p>

**git-trek** is a card-based, retro-futuristic TUI that transforms your git history into a navigable deck of commits. Scrub through time with left/right navigation while your working directory updates in real-time, letting you see and test your code at any point in history—without the fear of breaking anything.

## 🏁 Installation

### Prerequisites

The only requirement is the **Rust toolchain**:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Development (Recommended)

**Clone and run locally without touching your PATH:**

```bash
git clone https://github.com/junovhs/git-trek.git
cd git-trek
cargo run --release
```

This keeps everything isolated. No global installs, no PATH pollution. Perfect for hacking on git-trek or testing it out.

**To use from any directory**, create an alias in your shell config:

```bash
# Add to ~/.bashrc or ~/.zshrc
alias gt='/path/to/git-trek/target/release/git-trek'
```

### Production Install (When You're Ready to Ship)

**Only install globally when you're sure you want it permanently:**

```bash
cargo install --path .
```

This places the binary in `~/.cargo/bin/` and makes `git-trek` available everywhere.

**To uninstall:**

```bash
cargo uninstall git-trek
```

---

## 🎯 Features

### 🃏 Card-Based Navigation
Navigate through commits like flipping through SNES save file cards. Three commits visible at once: previous, current (highlighted), and next. Use **left/right arrows** or **A/D** to scrub through time.

### ⚡ Real-Time File Scrubbing
Your working directory updates as you navigate (with smart debouncing). Watch your editor change in real-time as you explore different points in history. No git commands to memorize—just arrow keys.

### 🛡️ Safe, Non-Destructive
Creates a hidden session branch to safely scrub through history. Your original branch stays untouched. Quit anytime with `Q` and you're instantly back to where you started.

### 🎨 Psychedelic Sci-Fi UI
Vibrant magenta, cyan, and yellow colors on pure black. The interface feels like a retro space console from 1989. Because exploring git history should look *cool*.

### 📊 Detailed Commit Inspection
Press `Enter` on any card to see full commit details: message, author, timestamp, and diff statistics. All presented in a clean, structured layout.

### 🌳 Dirty Tree Handling
Got uncommitted changes? No problem. git-trek offers three choices:
- **Stash** - Temporarily save changes (restored on exit)
- **Continue** - Browse in read-only mode (no checkout allowed)
- **Quit** - Exit without changes

No more "working tree dirty" errors blocking you.

---

## 🕹️ The Workflow

1. **Launch**: Run `cargo run --release` (or `git-trek` if installed)
2. **Browse**: Use **← →** or **A D** to flip through commit cards
3. **Watch**: Your files update in your editor ~200ms after you stop navigating
4. **Inspect**: Press **Enter** to see full commit details
5. **Checkout**: Press **C** from detail view to permanently checkout a commit
6. **Exit**: Press **Q** anytime to return to your original branch

---

## ⌨️ Controls

### Card View (Main)
| Key | Action |
|-----|--------|
| `←` `→` or `A` `D` | Navigate left/right through commit cards |
| `Enter` | Open detail view for current card |
| `P` | Pin anchor (marks current position) |
| `Q` | Quit and restore original branch |
| `?` | Show help |

### Detail View
| Key | Action |
|-----|--------|
| `Esc` or `Q` | Back to card view |
| `C` | Checkout this commit (with confirmation) |
| `T` | Toggle diff view |
| `P` / `F` | Mark test pass/fail (manual) |

### Checkout Confirmation
| Key | Action |
|-----|--------|
| `Y` | Confirm checkout (detaches HEAD) |
| `N` or `Esc` | Cancel and return to detail view |

---

## 🚨 Requirements

- Git repository
- Terminal with color support
- Rust toolchain (for building)

**Note:** git-trek handles dirty working trees gracefully—no need to commit or stash first.

---

## 🛠️ Development

### Quick Start
```bash
git clone https://github.com/junovhs/git-trek.git
cd git-trek
cargo run
```

### Build Release Binary
```bash
cargo build --release
# Binary is at: target/release/git-trek
```

### Run Tests
```bash
cargo test
cargo clippy
```

### Advanced Options
```bash
git-trek --autostash      # Auto-stash uncommitted changes
git-trek --worktree       # Use separate worktree (faster on large repos)
git-trek --since 2024-01-01  # Only show commits after date
git-trek --author "name"  # Filter by author
```

---

## 🎮 Pro Tips

- **Use `--release` for smooth navigation**: Debug builds make git operations slow
- **Run in large repos**: git-trek shines when you have hundreds of commits
- **Test different states**: Perfect for bisecting bugs or finding when features were added
- **Pin anchors**: Use `P` to mark important commits as you explore
- **Read-only mode**: Great for safely browsing history without checkout privileges

---

## 🐛 Troubleshooting

### "Navigation feels laggy"
Use `cargo run --release` instead of `cargo run`. Debug builds are 10x slower.

### "Files aren't updating"
Files update ~200ms after you **stop** navigating (debounced). This prevents the multi-line jump bug while keeping the scrubbing feature.

### "I want to reset everything"
```bash
cargo clean
rm -rf target
cargo build --release
```

---

*Made with spite and rust by developers who refuse to memorize git commands.*
```