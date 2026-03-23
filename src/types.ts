/** Grid position (row, col) */
export interface GridPos {
  row: number;
  col: number;
}

/** Desk info from the backend */
export interface DeskInfo {
  pos: GridPos;
  agent_id: string;
  has_agent: boolean;
}

/** Agent character state from the backend */
export interface AgentCharacter {
  agent_id: string;
  character_index: number;
  state: AgentStateTag;
  x: number;
  y: number;
  target_x: number;
  target_y: number;
  desk_pos: GridPos;
  direction: number; // 0=down, 1=up, 2=right, 3=left
  anim_frame: number;
  parent_agent_id: string | null;
  is_done: boolean;
}

/** Agent state discriminator */
export interface AgentStateTag {
  state:
    | "Entering"
    | "Idle"
    | "Walking"
    | "Typing"
    | "Reading"
    | "Meeting"
    | "Leaving"
    | "Gone"
    | "Fidgeting"
    | "WaterCooler"
    | "Whiteboard"
    | "Waving"
    | "Thinking"
    | "Celebrating"
    | "GettingCoffee";
}

/** Full office state emitted by the backend each tick */
export interface OfficeState {
  agents: AgentCharacter[];
  desks: DeskInfo[];
  grid_rows: number;
  grid_cols: number;
  door_pos: GridPos;
  meeting_pos: GridPos;
  water_cooler_pos: GridPos;
  coffee_machine_pos: GridPos;
  whiteboard_pos: GridPos;
  /** Monotonically increasing generation counter for change detection */
  generation: number;
}
