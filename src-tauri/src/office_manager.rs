use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use crate::parser::{classify_tool, TranscriptEvent};
use crate::state_machine::{grid_to_pixel, simple_hash, AgentCharacter, AgentState};

const DEFAULT_AGENT_ID: &str = "default-resident";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct GridPos {
    pub row: i32,
    pub col: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfficeState {
    pub agents: Vec<AgentCharacter>,
    pub desks: Vec<DeskInfo>,
    pub grid_rows: i32,
    pub grid_cols: i32,
    pub door_pos: GridPos,
    pub meeting_pos: GridPos,
    pub water_cooler_pos: GridPos,
    pub coffee_machine_pos: GridPos,
    pub whiteboard_pos: GridPos,
    /// Monotonically increasing generation counter; increments on every state change.
    pub generation: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeskInfo {
    pub pos: GridPos,
    pub agent_id: String,
    pub has_agent: bool,
}

/// Desk positions allocated on demand as agents arrive.
/// Row 0 is the door wall, so desks start at row 2+.
const DESK_POSITIONS: &[(i32, i32)] = &[
    (2, 2),
    (2, 5),
    (4, 2),
    (4, 5),
    (6, 2),
    (6, 5),
    (8, 2),
    (8, 5),
    (10, 2),
];

pub struct OfficeManager {
    agents: HashMap<String, AgentCharacter>,
    desks: Vec<DeskInfo>,
    grid_rows: i32,
    grid_cols: i32,
    door_pos: GridPos,
    meeting_pos: GridPos,
    water_cooler_pos: GridPos,
    coffee_machine_pos: GridPos,
    whiteboard_pos: GridPos,
    next_character_index: u8,
    next_desk_slot: usize,
    last_activity: HashMap<String, Instant>,
    pending_meetings: Vec<(String, String)>,
    active_meetings: Vec<(String, String)>,
    idle_timeout_secs: f64,
    /// Known parent relationships: child_agent_id -> parent_agent_id
    parent_map: HashMap<String, String>,
    /// Monotonic tick counter for pseudo-random seeding
    tick_counter: u64,
    /// Last time autonomous behaviors were checked (every ~5s)
    last_autonomous_check: Instant,
    /// Agents currently on autonomous walks (to prevent double-triggering)
    agents_on_break: std::collections::HashSet<String>,
    /// Cached BFS pathfinding results keyed by (from, to).
    /// Cleared when grid dimensions change.
    path_cache: HashMap<(GridPos, GridPos), Vec<GridPos>>,
    /// Grid dimensions when the path cache was last valid.
    path_cache_grid: (i32, i32),
    /// Monotonically increasing generation counter for change detection.
    generation: u64,
}

impl OfficeManager {
    pub fn new() -> Self {
        let grid_rows = 4;
        let grid_cols = 6;
        // Door on the back (top) wall, centered
        let door_pos = GridPos { row: 0, col: grid_cols / 2 };
        let meeting_pos = GridPos { row: 2, col: grid_cols / 2 };
        // Furniture positions: top-right area for water cooler, bottom-left for coffee, left wall for whiteboard
        let water_cooler_pos = GridPos { row: 1, col: grid_cols - 1 };
        let coffee_machine_pos = GridPos { row: 1, col: 0 };
        let whiteboard_pos = GridPos { row: 3, col: 0 };

        let mut mgr = Self {
            agents: HashMap::new(),
            desks: Vec::new(),
            grid_rows,
            grid_cols,
            door_pos,
            meeting_pos,
            water_cooler_pos,
            coffee_machine_pos,
            whiteboard_pos,
            next_character_index: 0,
            next_desk_slot: 0,
            last_activity: HashMap::new(),
            pending_meetings: Vec::new(),
            active_meetings: Vec::new(),
            idle_timeout_secs: 15.0,
            parent_map: HashMap::new(),
            tick_counter: 0,
            last_autonomous_check: Instant::now(),
            agents_on_break: std::collections::HashSet::new(),
            path_cache: HashMap::new(),
            path_cache_grid: (grid_rows, grid_cols),
            generation: 0,
        };
        mgr.spawn_default_agent();
        mgr
    }

    /// Spawns a single "resident" agent already seated at the first desk so the
    /// office feels alive on launch — no Claude Code activity required.
    fn spawn_default_agent(&mut self) {
        let char_index = self.next_character_index % 6;
        self.next_character_index += 1;

        let desk_pos = self.allocate_desk();

        let mut agent = AgentCharacter::new(
            DEFAULT_AGENT_ID.to_string(),
            char_index,
            desk_pos,
            self.door_pos,
            None,
        );
        // Place directly at desk in idle state (skip the enter-from-door walk)
        let desk_px = grid_to_pixel(desk_pos);
        agent.x = desk_px.0;
        agent.y = desk_px.1;
        agent.target_x = desk_px.0;
        agent.target_y = desk_px.1;
        agent.set_state(AgentState::Idle);
        agent.direction = 0;

        self.desks.push(DeskInfo {
            pos: desk_pos,
            agent_id: DEFAULT_AGENT_ID.to_string(),
            has_agent: true,
        });

        self.agents.insert(DEFAULT_AGENT_ID.to_string(), agent);
    }

    /// Cached BFS pathfinding. Returns a clone from cache or computes and caches.
    fn cached_path(&mut self, from: GridPos, to: GridPos) -> Vec<GridPos> {
        // Invalidate cache if grid dimensions changed
        let current_grid = (self.grid_rows, self.grid_cols);
        if current_grid != self.path_cache_grid {
            self.path_cache.clear();
            self.path_cache_grid = current_grid;
        }
        if let Some(cached) = self.path_cache.get(&(from, to)) {
            return cached.clone();
        }
        let path = find_path_on_grid(from, to, self.grid_rows, self.grid_cols);
        self.path_cache.insert((from, to), path.clone());
        path
    }

    pub fn handle_event(&mut self, event: TranscriptEvent) {
        match event {
            TranscriptEvent::SessionActive { ref agent_id, .. } => {
                self.last_activity.insert(agent_id.clone(), Instant::now());
                // Only auto-create the main agent
                if agent_id.starts_with("main-") && !self.agents.contains_key(agent_id) {
                    // Dismiss the default-resident when a real session agent arrives
                    if let Some(default_agent) = self.agents.get_mut(DEFAULT_AGENT_ID) {
                        let path = find_path_on_grid(
                            default_agent.desk_pos,
                            self.door_pos,
                            self.grid_rows,
                            self.grid_cols,
                        );
                        default_agent.start_leaving(path);
                    }
                    self.add_agent(agent_id.clone(), None);
                }
            }

            TranscriptEvent::SubAgentStarted {
                ref agent_id,
                ref parent_agent_id,
                ..
            } => {
                // A sub-agent JSONL file just appeared — register parent and create character
                self.parent_map.insert(agent_id.clone(), parent_agent_id.clone());
                self.last_activity.insert(agent_id.clone(), Instant::now());
                if !self.agents.contains_key(agent_id) {
                    self.add_agent(agent_id.clone(), Some(parent_agent_id.clone()));
                }
            }

            TranscriptEvent::ToolUse {
                ref agent_id,
                ref tool_name,
                ..
            } => {
                self.last_activity.insert(agent_id.clone(), Instant::now());

                // Sub-agents detected via ToolUse that we haven't seen yet
                if !self.agents.contains_key(agent_id) {
                    let parent = self.parent_map.get(agent_id).cloned();
                    self.add_agent(agent_id.clone(), parent);
                }

                if tool_name != "Agent" {
                    let tool_kind = classify_tool(tool_name);
                    if let Some(agent) = self.agents.get_mut(agent_id) {
                        // Allow tool use from Idle, Fidgeting, Thinking, or Celebrating
                        // (interrupts autonomous idle behaviors to get back to work)
                        match agent.state {
                            AgentState::Idle
                            | AgentState::Fidgeting
                            | AgentState::Thinking
                            | AgentState::Celebrating => {
                                agent.on_tool_use(tool_kind);
                            }
                            _ => {}
                        }
                    }
                    // Remove from break tracking if they were on one
                    self.agents_on_break.remove(agent_id);
                }
            }

            TranscriptEvent::ToolResult { ref agent_id, .. } => {
                self.last_activity.insert(agent_id.clone(), Instant::now());
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.on_tool_complete();
                }
            }

            TranscriptEvent::SubAgentSpawned {
                ref parent_agent_id, ..
            } => {
                self.last_activity.insert(parent_agent_id.clone(), Instant::now());
            }
        }
    }

    fn add_agent(&mut self, agent_id: String, parent_agent_id: Option<String>) {
        if self.agents.contains_key(&agent_id) {
            return;
        }

        let char_index = self.next_character_index % 6;
        self.next_character_index += 1;

        let desk_pos = self.allocate_desk();
        let path = self.cached_path(self.door_pos, desk_pos);

        let mut agent = AgentCharacter::new(
            agent_id.clone(),
            char_index,
            desk_pos,
            self.door_pos,
            parent_agent_id,
        );
        agent.start_walking(path);
        agent.state = AgentState::Entering;

        self.desks.push(DeskInfo {
            pos: desk_pos,
            agent_id: agent_id.clone(),
            has_agent: true,
        });

        self.agents.insert(agent_id, agent);
    }

    fn allocate_desk(&mut self) -> GridPos {
        let idx = self.next_desk_slot % DESK_POSITIONS.len();
        self.next_desk_slot += 1;

        let (row, col) = DESK_POSITIONS[idx];
        let pos = GridPos { row, col };

        let needed_rows = pos.row + 3;
        let needed_cols = pos.col + 2;
        if needed_rows > self.grid_rows {
            self.grid_rows = needed_rows;
        }
        if needed_cols > self.grid_cols {
            self.grid_cols = needed_cols;
        }

        self.meeting_pos = GridPos {
            row: self.grid_rows / 2,
            col: self.grid_cols / 2,
        };
        // Door stays on top wall, centered horizontally
        self.door_pos = GridPos {
            row: 0,
            col: self.grid_cols / 2,
        };

        pos
    }

    pub fn tick(&mut self, dt: f64) -> OfficeState {
        self.tick_counter += 1;
        self.generation += 1;

        let now = Instant::now();

        for agent in self.agents.values_mut() {
            agent.tick(dt, now);
        }

        self.check_idle_agents(now);
        self.check_fidgets(now);
        self.check_autonomous_behaviors(now);
        self.check_autonomous_return(now);
        self.check_waving(now);
        self.check_head_tracking();
        self.process_meetings();
        self.process_post_meeting(now);

        // Force-remove agents stuck in Gone or Leaving for too long (safety net)
        let mut force_gone: Vec<String> = Vec::new();
        for (id, agent) in &self.agents {
            if agent.state == AgentState::Leaving
                && now.duration_since(agent.state_start).as_secs_f64() > 5.0
            {
                force_gone.push(id.clone());
            }
        }
        for id in &force_gone {
            if let Some(agent) = self.agents.get_mut(id) {
                agent.set_state(AgentState::Gone);
            }
        }

        // Clean up gone agents
        let gone: Vec<String> = self
            .agents
            .iter()
            .filter(|(_, a)| a.state == AgentState::Gone)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &gone {
            self.agents.remove(id);
            self.agents_on_break.remove(id);
            self.last_activity.remove(id);
            for desk in &mut self.desks {
                if desk.agent_id == *id {
                    desk.has_agent = false;
                }
            }
        }

        self.get_state()
    }

    fn check_idle_agents(&mut self, now: Instant) {
        let timeout = self.idle_timeout_secs;
        let door_pos = self.door_pos;

        let mut to_leave: Vec<(String, GridPos)> = Vec::new();
        let mut to_meet: Vec<(String, String)> = Vec::new();

        for (id, last) in &self.last_activity {
            if now.duration_since(*last).as_secs_f64() > timeout {
                if let Some(agent) = self.agents.get(id) {
                    if !id.starts_with("main-")
                        && agent.state == AgentState::Idle
                        && !agent.is_done
                    {
                        let desk = agent.desk_pos;
                        if let Some(parent_id) = agent.parent_agent_id.clone() {
                            to_meet.push((id.clone(), parent_id));
                        } else {
                            to_leave.push((id.clone(), desk));
                        }
                    }
                }
            }
        }

        for (id, desk) in to_leave {
            let path = self.cached_path(desk, door_pos);
            if let Some(agent) = self.agents.get_mut(&id) {
                agent.is_done = true;
                agent.start_leaving(path);
            }
        }

        for (child_id, parent_id) in to_meet {
            if let Some(agent) = self.agents.get_mut(&child_id) {
                agent.is_done = true;
            }
            self.pending_meetings.push((child_id, parent_id));
        }
    }

    fn process_meetings(&mut self) {
        let meetings: Vec<(String, String)> = self.pending_meetings.drain(..).collect();
        let meeting_pos = self.meeting_pos;

        for (child_id, parent_id) in meetings {
            let child_can = self.agents.get(&child_id).map_or(false, |a| {
                a.state == AgentState::Idle && a.is_done
            });
            let parent_can = self.agents.get(&parent_id).map_or(false, |a| {
                matches!(a.state, AgentState::Idle | AgentState::Typing | AgentState::Reading
                    | AgentState::Fidgeting | AgentState::Thinking)
            });

            if child_can && parent_can {
                let child_desk = self.agents[&child_id].desk_pos;
                let parent_desk = self.agents[&parent_id].desk_pos;
                let child_path = self.cached_path(child_desk, meeting_pos);
                let parent_path = self.cached_path(parent_desk, meeting_pos);

                if let Some(child) = self.agents.get_mut(&child_id) {
                    child.start_meeting(child_path, parent_id.clone());
                }
                if let Some(parent) = self.agents.get_mut(&parent_id) {
                    parent.start_meeting(parent_path, child_id.clone());
                }

                self.active_meetings.push((child_id, parent_id));
            } else {
                self.pending_meetings.push((child_id, parent_id));
            }
        }
    }

    fn process_post_meeting(&mut self, now: Instant) {
        let meeting_pos = self.meeting_pos;
        let door_pos = self.door_pos;

        let mut completed = Vec::new();

        for (child_id, parent_id) in &self.active_meetings {
            let child_at = self.agents.get(child_id).map_or(false, |a| a.at_meeting_point);
            let parent_at = self.agents.get(parent_id).map_or(false, |a| a.at_meeting_point);

            if child_at && parent_at {
                let elapsed = self
                    .agents
                    .get(child_id)
                    .map_or(0.0, |a| now.duration_since(a.state_start).as_secs_f64());

                if elapsed > 3.0 {
                    completed.push((child_id.clone(), parent_id.clone()));
                }
            }
        }

        for (child_id, parent_id) in completed {
            self.active_meetings
                .retain(|(c, p)| !(c == &child_id && p == &parent_id));

            let parent_desk = self.agents.get(&parent_id).map(|a| a.desk_pos).unwrap_or(meeting_pos);
            let child_is_done = self.agents.get(&child_id).map_or(true, |a| a.is_done);
            let child_desk = self.agents.get(&child_id).map(|a| a.desk_pos).unwrap_or(meeting_pos);

            let parent_path = self.cached_path(meeting_pos, parent_desk);
            let child_path = if child_is_done {
                self.cached_path(meeting_pos, door_pos)
            } else {
                self.cached_path(meeting_pos, child_desk)
            };

            if let Some(parent) = self.agents.get_mut(&parent_id) {
                parent.start_walking(parent_path);
                parent.meeting_partner = None;
            }

            if let Some(child) = self.agents.get_mut(&child_id) {
                child.meeting_partner = None;
                if child_is_done {
                    child.start_leaving(child_path);
                } else {
                    child.start_walking(child_path);
                }
            }
        }
    }

    /// Check idle agents for fidget transitions.
    fn check_fidgets(&mut self, now: Instant) {
        let mut to_fidget: Vec<String> = Vec::new();

        for (id, agent) in &self.agents {
            if agent.state == AgentState::Idle && !agent.is_done {
                if let Some(idle_since) = agent.idle_since {
                    let idle_dur = now.duration_since(idle_since).as_secs_f64();
                    if idle_dur >= agent.fidget_threshold {
                        to_fidget.push(id.clone());
                    }
                }
            }
        }

        for id in to_fidget {
            if let Some(agent) = self.agents.get_mut(&id) {
                agent.set_state(AgentState::Fidgeting);
                agent.direction = 0; // face down during fidget
            }
        }
    }

    /// Check for autonomous behaviors (water cooler, coffee, whiteboard) every ~5 seconds.
    fn check_autonomous_behaviors(&mut self, now: Instant) {
        if now.duration_since(self.last_autonomous_check).as_secs_f64() < 5.0 {
            return;
        }
        self.last_autonomous_check = now;

        let water_cooler_pos = self.water_cooler_pos;
        let coffee_machine_pos = self.coffee_machine_pos;
        let whiteboard_pos = self.whiteboard_pos;
        let tick = self.tick_counter;

        // Collect idle agents (id + desk_pos) that are not on break and not done
        let idle_agents: Vec<(String, GridPos)> = self
            .agents
            .iter()
            .filter(|(id, a)| {
                a.state == AgentState::Idle
                    && !a.is_done
                    && !self.agents_on_break.contains(*id)
            })
            .map(|(id, a)| (id.clone(), a.desk_pos))
            .collect();

        let mut water_cooler_taken = false;
        let mut coffee_taken = false;
        // Collect actions: (agent_id, dest_state)
        let mut actions: Vec<(String, GridPos, AgentState)> = Vec::new();

        // Water cooler check: 5% chance per idle agent
        for (id, desk_pos) in &idle_agents {
            let hash = simple_hash(id, tick.wrapping_add(1000));
            if hash % 100 < 5 && !water_cooler_taken {
                actions.push((id.clone(), *desk_pos, AgentState::WaterCooler));
                water_cooler_taken = true;
            }
        }

        // Coffee run check: 3% chance per idle agent
        for (id, desk_pos) in &idle_agents {
            if actions.iter().any(|(aid, _, _)| aid == id) {
                continue;
            }
            let hash = simple_hash(id, tick.wrapping_add(2000));
            if hash % 100 < 3 && !coffee_taken {
                actions.push((id.clone(), *desk_pos, AgentState::GettingCoffee));
                coffee_taken = true;
            }
        }

        // Whiteboard check: if 2+ idle agents, small chance to start a whiteboard session
        let action_ids: std::collections::HashSet<&String> = actions.iter().map(|(id, _, _)| id).collect();
        let remaining_idle: Vec<&(String, GridPos)> = idle_agents
            .iter()
            .filter(|(id, _)| !action_ids.contains(id))
            .collect();

        if remaining_idle.len() >= 2 {
            let hash = simple_hash("whiteboard", tick);
            if hash % 100 < 8 {
                for (id, desk_pos) in remaining_idle.into_iter().take(2) {
                    actions.push((id.clone(), *desk_pos, AgentState::Whiteboard));
                }
            }
        }

        // Now compute paths and apply actions
        for (id, desk_pos, dest_state) in actions {
            let dest = match dest_state {
                AgentState::WaterCooler => water_cooler_pos,
                AgentState::GettingCoffee => coffee_machine_pos,
                AgentState::Whiteboard => GridPos {
                    row: whiteboard_pos.row,
                    col: whiteboard_pos.col + 1,
                },
                _ => unreachable!(),
            };
            let path = self.cached_path(desk_pos, dest);
            if let Some(agent) = self.agents.get_mut(&id) {
                agent.start_autonomous_walk(path, dest_state);
                self.agents_on_break.insert(id);
            }
        }
    }

    /// Return agents from autonomous destinations (water cooler, coffee, whiteboard) back to desk.
    fn check_autonomous_return(&mut self, now: Instant) {
        let mut to_return: Vec<(String, GridPos, GridPos)> = Vec::new();

        for (id, agent) in &self.agents {
            match agent.state {
                AgentState::WaterCooler => {
                    if agent.at_autonomous_dest
                        && now.duration_since(agent.state_start).as_secs_f64() > 3.5
                    {
                        let current_pos = pixel_to_nearest_grid(agent.x, agent.y);
                        to_return.push((id.clone(), current_pos, agent.desk_pos));
                    }
                }
                AgentState::GettingCoffee => {
                    if agent.at_autonomous_dest
                        && now.duration_since(agent.state_start).as_secs_f64() > 2.5
                    {
                        let current_pos = pixel_to_nearest_grid(agent.x, agent.y);
                        to_return.push((id.clone(), current_pos, agent.desk_pos));
                    }
                }
                AgentState::Whiteboard => {
                    if agent.at_autonomous_dest
                        && now.duration_since(agent.state_start).as_secs_f64() > 5.0
                    {
                        let current_pos = pixel_to_nearest_grid(agent.x, agent.y);
                        to_return.push((id.clone(), current_pos, agent.desk_pos));
                    }
                }
                _ => {}
            }
        }

        for (id, current_pos, desk) in to_return {
            let path = self.cached_path(current_pos, desk);
            if let Some(agent) = self.agents.get_mut(&id) {
                agent.start_walking(path);
            }
            self.agents_on_break.remove(&id);
        }
    }

    /// Check if entering/walking agents pass near seated agents (trigger waving).
    /// Uses a spatial grid for O(1) proximity lookups instead of O(N*M) nested loops.
    fn check_waving(&mut self, now: Instant) {
        // Build spatial grid of walking agents (96px cells — matches the larger proximity threshold)
        const CELL_SIZE: f64 = 96.0;
        let mut walker_grid: HashMap<(i32, i32), Vec<(String, f64, f64)>> = HashMap::new();
        let mut has_walkers = false;

        for (id, a) in &self.agents {
            if a.state == AgentState::Entering || a.state == AgentState::Walking {
                let cx = (a.x / CELL_SIZE) as i32;
                let cy = (a.y / CELL_SIZE) as i32;
                walker_grid.entry((cx, cy)).or_default().push((id.clone(), a.x, a.y));
                has_walkers = true;
            }
        }

        if !has_walkers {
            return;
        }

        // Check seated agents against nearby spatial cells
        let mut to_wave: Vec<(String, u8)> = Vec::new();

        for (id, agent) in &self.agents {
            let can_wave = matches!(
                agent.state,
                AgentState::Idle | AgentState::Fidgeting
            ) && !agent.is_done;

            if !can_wave {
                continue;
            }

            let cx = (agent.x / CELL_SIZE) as i32;
            let cy = (agent.y / CELL_SIZE) as i32;

            // Check own cell + 8 neighbors (3x3 around agent's cell)
            'outer: for dx in -1..=1 {
                for dy in -1..=1 {
                    if let Some(walkers) = walker_grid.get(&(cx + dx, cy + dy)) {
                        for (walker_id, wx, wy) in walkers {
                            if walker_id == id {
                                continue;
                            }
                            let dist_y = (agent.y - wy).abs();
                            let dist_x = (agent.x - wx).abs();
                            if dist_y < 48.0 && dist_x < 96.0 {
                                let dir = if (wx - agent.x).abs() > (wy - agent.y).abs() {
                                    if *wx > agent.x { 2 } else { 3 }
                                } else {
                                    if *wy > agent.y { 0 } else { 1 }
                                };
                                to_wave.push((id.clone(), dir));
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        for (id, dir) in to_wave {
            if let Some(agent) = self.agents.get_mut(&id) {
                agent.start_waving(dir);
            }
        }

        // Also: agents at water cooler wave at each other
        let cooler_agents: Vec<String> = self
            .agents
            .iter()
            .filter(|(_, a)| a.state == AgentState::WaterCooler && a.at_autonomous_dest)
            .map(|(id, _)| id.clone())
            .collect();

        if cooler_agents.len() >= 2 {
            for id in &cooler_agents {
                if let Some(agent) = self.agents.get(id) {
                    let el = now.duration_since(agent.state_start).as_secs_f64();
                    if el > 1.0 && el < 1.1
                    {
                        // Brief window to trigger wave — handled by tick timing
                    }
                }
            }
        }
    }

    /// Head tracking: seated agents turn to face walking agents passing nearby.
    fn check_head_tracking(&mut self) {
        // Collect walking agent positions
        let walking_agents: Vec<(String, f64, f64)> = self
            .agents
            .iter()
            .filter(|(_, a)| {
                matches!(
                    a.state,
                    AgentState::Entering
                        | AgentState::Walking
                        | AgentState::Leaving
                        | AgentState::WaterCooler
                        | AgentState::GettingCoffee
                )
                    && !a.at_autonomous_dest // only while actually moving
            })
            .map(|(id, a)| (id.clone(), a.x, a.y))
            .collect();

        if walking_agents.is_empty() {
            return;
        }

        // Check seated agents
        for (id, agent) in self.agents.iter_mut() {
            let is_seated = matches!(
                agent.state,
                AgentState::Idle | AgentState::Typing | AgentState::Reading
            );
            if !is_seated {
                continue;
            }

            let mut closest_dist = f64::MAX;
            let mut closest_dir: Option<u8> = None;

            for (walker_id, wx, wy) in &walking_agents {
                if walker_id == id {
                    continue;
                }
                let dx = wx - agent.x;
                let dy = wy - agent.y;
                let dist = (dx * dx + dy * dy).sqrt();

                // Only track if within ~3 tiles (96 pixels)
                if dist < 96.0 && dist < closest_dist {
                    closest_dist = dist;
                    // Face toward the walker
                    if dx.abs() > dy.abs() {
                        closest_dir = Some(if dx > 0.0 { 2 } else { 3 });
                    } else {
                        closest_dir = Some(if dy > 0.0 { 0 } else { 1 });
                    }
                }
            }

            if let Some(dir) = closest_dir {
                agent.direction = dir;
            }
            // If no walker nearby, direction stays as is (will reset to 0 on next state change)
        }
    }

    pub fn end_session(&mut self) {
        let door_pos = self.door_pos;

        let agents_to_leave: Vec<(String, GridPos)> = self
            .agents
            .iter()
            .filter(|(id, a)| {
                id.as_str() != DEFAULT_AGENT_ID
                    && a.state != AgentState::Leaving
                    && a.state != AgentState::Gone
            })
            .map(|(id, a)| (id.clone(), pixel_to_nearest_grid(a.x, a.y)))
            .collect();

        for (id, current_pos) in agents_to_leave {
            let path = self.cached_path(current_pos, door_pos);
            if let Some(agent) = self.agents.get_mut(&id) {
                agent.start_leaving(path);
            }
        }
    }

    pub fn get_state(&self) -> OfficeState {
        OfficeState {
            agents: self.agents.values().cloned().collect(),
            desks: self.desks.clone(),
            grid_rows: self.grid_rows,
            grid_cols: self.grid_cols,
            door_pos: self.door_pos,
            meeting_pos: self.meeting_pos,
            water_cooler_pos: self.water_cooler_pos,
            coffee_machine_pos: self.coffee_machine_pos,
            whiteboard_pos: self.whiteboard_pos,
            generation: self.generation,
        }
    }
}

/// Convert pixel position back to the nearest grid position.
pub fn pixel_to_nearest_grid(x: f64, y: f64) -> GridPos {
    use crate::state_machine::TILE_SIZE;
    GridPos {
        row: (y / TILE_SIZE).round() as i32,
        col: (x / TILE_SIZE).round() as i32,
    }
}

pub fn find_path_on_grid(from: GridPos, to: GridPos, rows: i32, cols: i32) -> Vec<GridPos> {
    if from == to {
        return vec![];
    }

    let mut visited: HashMap<GridPos, GridPos> = HashMap::new();
    let mut queue = VecDeque::new();
    visited.insert(from, from);
    queue.push_back(from);

    let directions = [
        GridPos { row: -1, col: 0 },
        GridPos { row: 1, col: 0 },
        GridPos { row: 0, col: -1 },
        GridPos { row: 0, col: 1 },
    ];

    while let Some(current) = queue.pop_front() {
        if current == to {
            let mut path = Vec::new();
            let mut pos = to;
            while pos != from {
                path.push(pos);
                pos = visited[&pos];
            }
            path.reverse();
            return path;
        }

        for dir in &directions {
            let next = GridPos {
                row: current.row + dir.row,
                col: current.col + dir.col,
            };
            if next.row < 0 || next.row >= rows || next.col < 0 || next.col >= cols {
                continue;
            }
            if visited.contains_key(&next) {
                continue;
            }
            visited.insert(next, current);
            queue.push_back(next);
        }
    }

    vec![to]
}
