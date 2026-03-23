use serde::Serialize;
use std::time::Instant;

use crate::office_manager::GridPos;
use crate::parser::ToolKind;

/// The states a character (agent) can be in.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state")]
pub enum AgentState {
    /// Walking from door to assigned desk
    Entering,
    /// Sitting at desk, no active tool
    Idle,
    /// Walking along a path to a target
    Walking,
    /// Actively using a writing tool at desk
    Typing,
    /// Actively using a reading tool at desk
    Reading,
    /// Walking to or at the meeting point for inter-agent communication
    Meeting,
    /// Walking from desk to door, about to exit
    Leaving,
    /// Has exited the office (not rendered)
    Gone,
    /// Idle micro-animation: stretching, looking around, sipping coffee
    Fidgeting,
    /// Walking to or standing at the water cooler
    WaterCooler,
    /// Standing at the whiteboard with other agents
    Whiteboard,
    /// Briefly waving at a passing or nearby agent
    Waving,
    /// Pondering after a tool completes (thought bubble)
    Thinking,
    /// Brief celebration after a long tool sequence
    Celebrating,
    /// Walking to or standing at the coffee machine
    GettingCoffee,
}

/// Represents one agent character in the office.
#[derive(Debug, Clone, Serialize)]
pub struct AgentCharacter {
    /// Unique agent ID (e.g., "main-<uuid>" or "agent-a5c35...")
    pub agent_id: String,
    /// Which character sprite to use (0-5)
    pub character_index: u8,
    /// Current state
    pub state: AgentState,
    /// Current pixel position (for smooth rendering)
    pub x: f64,
    pub y: f64,
    /// Target pixel position (for interpolation)
    pub target_x: f64,
    pub target_y: f64,
    /// Path of grid positions to walk
    #[serde(skip)]
    pub path: Vec<GridPos>,
    /// Index into path
    #[serde(skip)]
    pub path_index: usize,
    /// Assigned desk grid position
    pub desk_pos: GridPos,
    /// Direction the character is facing (0=down, 1=up, 2=right, 3=left)
    pub direction: u8,
    /// Animation frame index
    pub anim_frame: u8,
    /// When the current state started
    #[serde(skip)]
    pub state_start: Instant,
    /// Parent agent ID (for sub-agents)
    pub parent_agent_id: Option<String>,
    /// Whether the agent has been flagged as done
    pub is_done: bool,
    /// Meeting partner agent ID (when in Meeting state)
    #[serde(skip)]
    pub meeting_partner: Option<String>,
    /// Whether this agent has arrived at the meeting point
    #[serde(skip)]
    pub at_meeting_point: bool,
    /// Number of tools used since this agent entered
    pub tool_count: u16,
    /// When the agent last became Idle (for fidget timing)
    #[serde(skip)]
    pub idle_since: Option<Instant>,
    /// State to restore after Waving finishes
    #[serde(skip)]
    pub pre_wave_state: Option<AgentState>,
    /// Direction to restore after head tracking
    #[serde(skip)]
    pub pre_wave_direction: u8,
    /// Whether this agent has arrived at an autonomous-walk destination
    /// (water cooler, coffee machine, whiteboard)
    #[serde(skip)]
    pub at_autonomous_dest: bool,
    /// Random fidget threshold in seconds (3-8), computed per idle entry
    #[serde(skip)]
    pub fidget_threshold: f64,
}

impl AgentCharacter {
    pub fn new(
        agent_id: String,
        character_index: u8,
        desk_pos: GridPos,
        door_pos: GridPos,
        parent_agent_id: Option<String>,
    ) -> Self {
        // Start at the door position
        let door_px = grid_to_pixel(door_pos);
        Self {
            agent_id,
            character_index,
            state: AgentState::Entering,
            x: door_px.0,
            y: door_px.1,
            target_x: door_px.0,
            target_y: door_px.1,
            path: Vec::new(),
            path_index: 0,
            desk_pos,
            direction: 0,
            anim_frame: 0,
            state_start: Instant::now(),
            parent_agent_id,
            is_done: false,
            meeting_partner: None,
            at_meeting_point: false,
            tool_count: 0,
            idle_since: None,
            pre_wave_state: None,
            pre_wave_direction: 0,
            at_autonomous_dest: false,
            fidget_threshold: 5.0,
        }
    }

    /// Transition to a new state.
    pub fn set_state(&mut self, state: AgentState) {
        if state == AgentState::Idle {
            let now = Instant::now();
            self.idle_since = Some(now);
            // Compute a pseudo-random fidget threshold (3.0-8.0s) based on agent_id hash + time
            let hash = simple_hash(&self.agent_id, now.elapsed().as_nanos() as u64);
            self.fidget_threshold = 3.0 + (hash % 500) as f64 / 100.0; // 3.0 - 8.0
        } else {
            self.idle_since = None;
        }
        self.state = state;
        self.state_start = Instant::now();
        self.anim_frame = 0;
        self.at_meeting_point = false;
        self.at_autonomous_dest = false;
    }

    /// Start walking along a path.
    pub fn start_walking(&mut self, path: Vec<GridPos>) {
        if path.is_empty() {
            return;
        }
        self.path = path;
        self.path_index = 0;
        self.set_state(AgentState::Walking);
        self.advance_to_next_waypoint();
    }

    /// Start walking to meeting point.
    pub fn start_meeting(&mut self, path: Vec<GridPos>, partner_id: String) {
        self.meeting_partner = Some(partner_id);
        self.at_meeting_point = false;
        if path.is_empty() {
            self.set_state(AgentState::Meeting);
            self.at_meeting_point = true;
            return;
        }
        self.path = path;
        self.path_index = 0;
        self.set_state(AgentState::Meeting);
        self.advance_to_next_waypoint();
    }

    /// Start leaving (walk to door).
    pub fn start_leaving(&mut self, path: Vec<GridPos>) {
        if path.is_empty() {
            self.set_state(AgentState::Gone);
            return;
        }
        self.path = path;
        self.path_index = 0;
        self.set_state(AgentState::Leaving);
        self.advance_to_next_waypoint();
    }

    /// Called on tool use to animate at desk.
    pub fn on_tool_use(&mut self, tool_kind: ToolKind) {
        self.tool_count += 1;
        match tool_kind {
            ToolKind::Reading => self.set_state(AgentState::Reading),
            ToolKind::Writing => self.set_state(AgentState::Typing),
        }
    }

    /// Called when tool completes.
    pub fn on_tool_complete(&mut self) {
        if self.state == AgentState::Typing || self.state == AgentState::Reading {
            // Celebration if > 5 tools used since entering
            if self.tool_count > 5 {
                self.set_state(AgentState::Celebrating);
                self.tool_count = 0; // reset after celebration
                self.direction = 0; // face down
            } else {
                // Thinking state for 1-3 seconds before going idle
                self.set_state(AgentState::Thinking);
                self.direction = 0;
            }
        }
    }

    /// Update character position each tick. Returns true if still moving.
    /// `now` is a cached Instant for the current tick to avoid repeated syscalls.
    pub fn tick(&mut self, dt: f64, now: Instant) -> bool {
        let speed = 120.0; // pixels per second

        // Animate frames
        let elapsed = now.duration_since(self.state_start).as_secs_f64();
        self.anim_frame = ((elapsed * 4.0) as u8) % 4; // 4 fps animation

        // Handle timed state transitions
        match self.state {
            AgentState::Fidgeting => {
                if elapsed > 2.0 {
                    self.set_state(AgentState::Idle);
                    self.direction = 0;
                    return false;
                }
            }
            AgentState::Waving => {
                if elapsed > 1.5 {
                    // Restore previous state
                    if let Some(prev) = self.pre_wave_state.take() {
                        self.state = prev;
                        self.state_start = now;
                        self.direction = self.pre_wave_direction;
                    } else {
                        self.set_state(AgentState::Idle);
                        self.direction = 0;
                    }
                    return false;
                }
            }
            AgentState::Thinking => {
                // Stay for 1-3 seconds (use pseudo-random from agent_id)
                let elapsed_nanos = now.duration_since(self.state_start).as_nanos() as u64;
                let think_dur = 1.0 + (simple_hash(&self.agent_id, elapsed_nanos / 1_000_000_000) % 200) as f64 / 100.0;
                if elapsed > think_dur {
                    self.set_state(AgentState::Idle);
                    self.direction = 0;
                    return false;
                }
            }
            AgentState::Celebrating => {
                if elapsed > 1.5 {
                    self.set_state(AgentState::Idle);
                    self.direction = 0;
                    return false;
                }
            }
            AgentState::WaterCooler => {
                // Standing at the water cooler for 3-4 seconds
                if self.at_autonomous_dest && elapsed > 3.5 {
                    // Walk back to desk — the office_manager will handle this
                    return false;
                }
            }
            AgentState::GettingCoffee => {
                // Standing at coffee machine for 2-3 seconds
                if self.at_autonomous_dest && elapsed > 2.5 {
                    return false;
                }
            }
            AgentState::Whiteboard => {
                // Standing at whiteboard for 4-6 seconds
                if self.at_autonomous_dest && elapsed > 5.0 {
                    return false;
                }
            }
            _ => {}
        }

        // Move toward target
        let dx = self.target_x - self.x;
        let dy = self.target_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 1.0 {
            self.x = self.target_x;
            self.y = self.target_y;

            // Advance to next waypoint if walking/entering/leaving/meeting or autonomous walks
            match self.state {
                AgentState::Entering
                | AgentState::Walking
                | AgentState::Leaving
                | AgentState::Meeting
                | AgentState::WaterCooler
                | AgentState::GettingCoffee
                | AgentState::Whiteboard => {
                    if self.path_index < self.path.len() {
                        self.advance_to_next_waypoint();
                        return true;
                    }
                    // Path complete — handle state transition
                    self.on_path_complete();
                }
                _ => {}
            }
            return false;
        }

        // Move toward target
        let step = speed * dt;
        let ratio = step / dist;
        self.x += dx * ratio.min(1.0);
        self.y += dy * ratio.min(1.0);

        // Update direction based on movement
        if dx.abs() > dy.abs() {
            self.direction = if dx > 0.0 { 2 } else { 3 }; // right or left
        } else {
            self.direction = if dy > 0.0 { 0 } else { 1 }; // down or up
        }

        true
    }

    fn advance_to_next_waypoint(&mut self) {
        if self.path_index < self.path.len() {
            let next = self.path[self.path_index];
            let px = grid_to_pixel(next);
            self.target_x = px.0;
            self.target_y = px.1;
            self.path_index += 1;
        }
    }

    fn on_path_complete(&mut self) {
        match self.state {
            AgentState::Entering | AgentState::Walking => {
                self.set_state(AgentState::Idle);
                // Face down when sitting at desk
                self.direction = 0;
            }
            AgentState::Meeting => {
                self.at_meeting_point = true;
                // Face toward meeting partner (default: face down)
                self.direction = 0;
            }
            AgentState::Leaving => {
                self.set_state(AgentState::Gone);
            }
            AgentState::WaterCooler => {
                self.at_autonomous_dest = true;
                self.state_start = Instant::now(); // reset timer for standing duration
                self.direction = 1; // face up (toward wall/cooler)
            }
            AgentState::GettingCoffee => {
                self.at_autonomous_dest = true;
                self.state_start = Instant::now();
                self.direction = 1; // face up (toward coffee machine)
            }
            AgentState::Whiteboard => {
                self.at_autonomous_dest = true;
                self.state_start = Instant::now();
                self.direction = 3; // face left (toward whiteboard on left wall)
            }
            _ => {}
        }
    }

    /// Start walking to an autonomous destination (water cooler, coffee, whiteboard).
    pub fn start_autonomous_walk(&mut self, path: Vec<GridPos>, dest_state: AgentState) {
        if path.is_empty() {
            self.set_state(dest_state);
            self.at_autonomous_dest = true;
            return;
        }
        self.path = path;
        self.path_index = 0;
        self.set_state(dest_state);
        self.at_autonomous_dest = false;
        self.advance_to_next_waypoint();
    }

    /// Start waving — saves current state to restore later.
    pub fn start_waving(&mut self, face_direction: u8) {
        self.pre_wave_state = Some(self.state.clone());
        self.pre_wave_direction = self.direction;
        self.state = AgentState::Waving;
        self.state_start = Instant::now();
        self.anim_frame = 0;
        self.direction = face_direction;
    }
}

/// Grid tile size in pixels.
pub const TILE_SIZE: f64 = 32.0;

/// Convert grid position to pixel center.
pub fn grid_to_pixel(pos: GridPos) -> (f64, f64) {
    (
        pos.col as f64 * TILE_SIZE + TILE_SIZE / 2.0,
        pos.row as f64 * TILE_SIZE + TILE_SIZE / 2.0,
    )
}

/// Simple deterministic pseudo-random hash (no `rand` crate).
/// Uses a combination of string hash and a seed value for variety.
pub fn simple_hash(s: &str, seed: u64) -> u64 {
    let mut hash: u64 = seed.wrapping_add(5381);
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    // Mix bits
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x45d9f3b);
    hash ^= hash >> 16;
    hash
}
