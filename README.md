# CLI Agent Office (Tauri)

A cross-platform desktop app that visualizes Claude Code activity as animated pixel-art office workers. Each agent and sub-agent is represented by a unique character at their own desk.

Built with Tauri 2 (Rust backend + TypeScript/HTML5 Canvas frontend).

There is also a native [macOS version](https://github.com/calvintirrell/CLI_Office_Agent) (Swift/AppKit) with the same features.

Inspired by [Pixel Agents](https://github.com/pablodelucca/pixel-agents).

## How It Works

The app watches Claude Code's JSONL transcript files (`~/.claude/projects/`) in real time. A default resident agent sits at a desk when the app launches. When a Claude Code session starts, the resident leaves and the session's agent character enters the office and sits at their own desk. Sub-agents spawned via the Agent tool get their own desks too. Characters animate based on what Claude is doing — typing when writing code, reading when searching files, thinking after tool calls, and celebrating after streaks of activity.

## Features

### 14-State Character System

| State | Description | Duration/Trigger |
|-------|-------------|-----------------|
| **Entering** | Walks from door to assigned desk | On session start |
| **Idle** | Sits at desk, subtle bobbing | Default resting state |
| **Walking** | Traverses BFS-computed path | Between locations |
| **Typing** | Arm animation at desk | Writing tools (Edit, Write, Bash, etc.) |
| **Reading** | Arms-up focused posture | Reading tools (Read, Grep, Glob, etc.) |
| **Thinking** | Hand-on-chin pose, thought bubble | 1–3s after tool completes |
| **Celebrating** | Arms raised, jump | After 5+ consecutive tool uses |
| **Fidgeting** | Stretch/look-around animation | After 3–8s idle (random threshold) |
| **Water Cooler** | Walks to cooler, stands | 3.5s at cooler |
| **Getting Coffee** | Walks to machine, holds cup | 2.5s at machine |
| **Whiteboard** | Walks to board, points | 5.0s at board (requires 2+ idle agents) |
| **Waving** | Brief raised-arm wave | 1.5s, triggered by nearby walking agent |
| **Meeting** | Both agents walk to center rug | 3.0s with speech bubbles |
| **Leaving** | Walks from desk to door | On idle timeout or session end |

### 6 Diverse Procedural Characters

| # | Skin | Hair | Shirt |
|---|------|------|-------|
| 0 | Light brown | Black | Blue |
| 1 | Dark brown | Black curly | Red |
| 2 | Fair | Brown | Green |
| 3 | Olive | Auburn | Purple |
| 4 | Deep brown | Black | Orange |
| 5 | Peach | Blonde | Teal |

Each character has **224 sprites**: 14 states × 4 directions × 4 animation frames, generated procedurally and cached.

### Autonomous Idle Behaviors

Every 5 seconds, idle agents have a chance to:

- **Water cooler break** — 5% chance per agent, stand for 3.5s
- **Coffee run** — 3% chance per agent, stand for 2.5s
- **Whiteboard session** — 8% chance if 2+ idle agents, stand for 5.0s

Randomization uses a deterministic `simpleHash(agent_id, tick_counter)` for varied but reproducible behavior.

### Social Interactions

- **Waving** — Seated agents wave at walking agents within 96×48 pixel proximity
- **Head tracking** — Seated agents turn to face nearby walkers within 96px
- **Meetings** — Sub-agents meet their parent agent at the center rug before exiting, with speech bubbles

### Tool Classification

| Category | Tools |
|----------|-------|
| **Reading** (arms-up) | Read, Grep, Glob, WebFetch, WebSearch, ToolSearch, TaskList, TaskGet, LSP |
| **Writing** (typing) | Edit, Write, Bash, TaskCreate, TaskUpdate, and all others |
| **Special** | Agent tool triggers sub-agent spawn (no animation on parent) |

### Office Layout & Furniture

- **Door** — Top wall, centered horizontally (agent entry/exit)
- **Water cooler** — Top-right corner (row 1, last column)
- **Coffee machine** — Top-left corner (row 1, column 0)
- **Whiteboard** — Left wall (row 3, column 0) with colored marker lines
- **Meeting area** — Center rug with concentric ellipses
- **Desks** — Up to 9 allocated on demand at predefined grid positions, each with monitor, keyboard, coffee mug, and rolling office chair
- **Rendering** — Checkerboard floor, gradient walls with night-sky windows, radial vignette lighting

### Session Lifecycle

- **App launch** — Default resident agent seated at first desk
- **Session starts** — Resident walks to door and leaves; session agent enters from door, walks to desk
- **Sub-agent spawned** — New character enters from door, gets own desk
- **Sub-agent idle 15s** — Walks to center rug to meet parent agent, then leaves
- **Session idle 15s** — Non-main agents leave; main agents stay
- **End session** — All non-default agents walk to door and leave

### Default Resident Dismissal

When a real Claude Code session agent arrives, the default resident automatically walks to the door and exits. This keeps the office clean — only active session agents are visible during use. When all sessions end, a new default resident spawns.

## Prerequisites

- [Node.js](https://nodejs.org/) 18+ and [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- Platform build tools (Xcode Command Line Tools on macOS, build-essential on Linux, Visual Studio C++ on Windows)

## Building

```bash
# Install frontend dependencies
pnpm install

# Development mode (hot-reload)
pnpm tauri dev

# Production build
pnpm tauri build
```

The production build outputs a platform-native app bundle in `src-tauri/target/release/bundle/`.

## Auto-Launch with Claude Code

The included `hooks/launch-office.sh` script can automatically open the app when any Claude Code session starts.

Add this to your `~/.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/cli-agent-office-tauri/hooks/launch-office.sh"
          }
        ]
      }
    ]
  }
}
```

Replace `/path/to/cli-agent-office-tauri` with the actual path where you cloned this repo. The script checks if the app is already running and only opens it if not.

## Architecture

```
Claude Code JSONL transcripts (~/.claude/projects/)
        |
        v
watcher.rs -- notify file watcher + scan_and_replay_active()
        |
        v
parser.rs -- JSONL parsing, tool classification
        |
        v
office_manager.rs -- Grid, desks, pathfinding, meetings,
        |              autonomous behaviors, head tracking, session lifecycle
        v
state_machine.rs -- Per-agent 14-state machine, movement, sprite frames
        |
        v
lib.rs -- Tauri commands, IPC bridge (get_render_state)
        |
        v
renderer.ts -- HTML5 Canvas 2D rendering (floor, walls, furniture, agents, lighting)
sprites.ts -- Procedural character sprites (224 per character, cached)
        |
        v
main.ts -- 60fps render loop, Tauri invoke calls
index.html -- Canvas element (640×480)
```

## Project Structure

```
cli-agent-office-tauri/
  src/
    main.ts                  # App entry, 60fps render loop
    renderer.ts              # Canvas 2D rendering
    sprites.ts               # Procedural sprite generation
    types.ts                 # TypeScript type definitions
    styles.css               # Minimal styling
    assets/                  # Static assets
  src-tauri/
    src/
      main.rs                # Tauri entry point
      lib.rs                 # Tauri commands, IPC bridge
      office_manager.rs      # Grid, desks, pathfinding, agent lifecycle
      state_machine.rs       # Per-agent state machine (14 states)
      watcher.rs             # File monitoring (notify crate)
      parser.rs              # JSONL parsing, tool classification
    Cargo.toml               # Rust dependencies
    tauri.conf.json           # Tauri app configuration
    capabilities/            # Tauri permission capabilities
    icons/                   # App icons (all platforms)
  package.json               # Frontend dependencies
  vite.config.ts             # Vite bundler config
  tsconfig.json              # TypeScript config
  hooks/launch-office.sh     # SessionStart hook for Claude Code
```

## Key Constants

| Constant | Value |
|----------|-------|
| Tile size | 32px |
| Default grid | 4 rows × 6 columns (grows dynamically) |
| Movement speed | 120px/s |
| Manager tick rate | ~30fps |
| Render rate | 60fps |
| Animation frame rate | 4fps |
| Idle timeout | 15s |
| Autonomous check interval | 5s |
| Watcher poll interval | 1s |
| Max desk slots | 9 |

## License

MIT — see [LICENSE](LICENSE).
