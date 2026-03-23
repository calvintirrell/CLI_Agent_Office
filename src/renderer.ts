import type { OfficeState, AgentCharacter } from "./types";
import {
  generateCharacterSprites,
  drawDesk,
  drawDoor,
  drawMeetingArea,
  drawSpeechBubble,
  drawThoughtBubble,
  drawWaterCooler,
  drawCoffeeMachine,
  drawWhiteboard,
  CHAR_SIZE,
} from "./sprites";

const TILE_SIZE = 32;

const spriteCache = new Map<number, Map<string, HTMLCanvasElement>>();
let animTime = 0;

// Gradient cache — recreated only when canvas dimensions change
let cachedGradients: {
  width: number;
  height: number;
  bgGrad: CanvasGradient;
  barGrad: CanvasGradient;
} | null = null;

function getCachedGradients(ctx: CanvasRenderingContext2D, w: number, h: number) {
  if (cachedGradients && cachedGradients.width === w && cachedGradients.height === h) {
    return cachedGradients;
  }
  const bgGrad = ctx.createLinearGradient(0, 0, 0, h);
  bgGrad.addColorStop(0, "#151525");
  bgGrad.addColorStop(1, "#1a1a30");

  const barGrad = ctx.createLinearGradient(0, h - 20, 0, h);
  barGrad.addColorStop(0, "rgba(0,0,0,0.6)");
  barGrad.addColorStop(1, "rgba(0,0,0,0.8)");

  cachedGradients = { width: w, height: h, bgGrad, barGrad };
  return cachedGradients;
}

function getSprites(charIndex: number): Map<string, HTMLCanvasElement> {
  if (!spriteCache.has(charIndex)) {
    spriteCache.set(charIndex, generateCharacterSprites(charIndex));
  }
  return spriteCache.get(charIndex)!;
}

function getScale(canvas: HTMLCanvasElement, state: OfficeState): number {
  const displayW = canvas.clientWidth;
  const displayH = canvas.clientHeight - 20;
  const gridW = state.grid_cols * TILE_SIZE;
  const gridH = state.grid_rows * TILE_SIZE;
  const scaleX = displayW / gridW;
  const scaleY = displayH / gridH;
  return Math.min(scaleX, scaleY, 4) * 0.88;
}

export function render(
  ctx: CanvasRenderingContext2D,
  canvas: HTMLCanvasElement,
  state: OfficeState | null
): void {
  const dpr = window.devicePixelRatio || 1;
  const displayWidth = canvas.clientWidth;
  const displayHeight = canvas.clientHeight;

  if (canvas.width !== displayWidth * dpr || canvas.height !== displayHeight * dpr) {
    canvas.width = displayWidth * dpr;
    canvas.height = displayHeight * dpr;
  }

  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  animTime = (animTime + 0.016) % (Math.PI * 2000);

  // Background (use cached gradient)
  const grads = getCachedGradients(ctx, displayWidth, displayHeight);
  ctx.fillStyle = grads.bgGrad;
  ctx.fillRect(0, 0, displayWidth, displayHeight);

  if (!state) {
    drawWaitingScreen(ctx, displayWidth, displayHeight);
    return;
  }

  const scale = getScale(canvas, state);
  const tileSize = TILE_SIZE * scale;

  const gridWidth = state.grid_cols * tileSize;
  const gridHeight = state.grid_rows * tileSize;
  const offsetX = (displayWidth - gridWidth) / 2;
  const offsetY = (displayHeight - 20 - gridHeight) / 2;

  ctx.save();
  ctx.translate(offsetX, offsetY);
  ctx.imageSmoothingEnabled = false;

  // Floor
  drawFloor(ctx, state, tileSize, scale);

  // Walls
  drawWalls(ctx, state, tileSize, scale);

  // Meeting area
  const mx = state.meeting_pos.col * tileSize;
  const my = state.meeting_pos.row * tileSize;
  drawMeetingArea(ctx, mx, my, tileSize);

  // Water cooler
  const wcx = state.water_cooler_pos.col * tileSize;
  const wcy = state.water_cooler_pos.row * tileSize;
  drawWaterCooler(ctx, wcx, wcy, scale);

  // Coffee machine
  const cmx = state.coffee_machine_pos.col * tileSize;
  const cmy = state.coffee_machine_pos.row * tileSize;
  drawCoffeeMachine(ctx, cmx, cmy, scale);

  // Whiteboard
  const wbx = state.whiteboard_pos.col * tileSize;
  const wby = state.whiteboard_pos.row * tileSize;
  drawWhiteboard(ctx, wbx, wby, scale);

  // Door
  const dx = state.door_pos.col * tileSize;
  const dy = state.door_pos.row * tileSize;
  drawDoor(ctx, dx + 2 * scale, dy - tileSize * 0.3, scale);

  // Desks (sorted by row for z-ordering)
  const sortedDesks = [...state.desks].sort((a, b) => a.pos.row - b.pos.row);
  for (const desk of sortedDesks) {
    const deskX = desk.pos.col * tileSize - 4 * scale;
    const deskY = desk.pos.row * tileSize - 8 * scale;
    drawDesk(ctx, deskX, deskY, scale, desk.has_agent);
  }

  // Agents (sorted by y for z-ordering)
  const sortedAgents = [...state.agents].sort((a, b) => a.y - b.y);
  for (const agent of sortedAgents) {
    drawAgent(ctx, agent, scale);
  }

  // Overhead lighting effect
  drawLighting(ctx, state, tileSize);

  ctx.restore();

  // Status bar
  drawStatusBar(ctx, state, displayWidth, displayHeight);
}

function drawWaitingScreen(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number
): void {
  // Pulsing dot
  const pulse = Math.sin(animTime * 2) * 0.3 + 0.7;
  ctx.fillStyle = `rgba(90, 160, 255, ${pulse * 0.6})`;
  ctx.beginPath();
  ctx.arc(w / 2, h / 2 - 20, 4, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = `rgba(200, 200, 220, ${pulse})`;
  ctx.font = "11px -apple-system, sans-serif";
  ctx.textAlign = "center";
  ctx.fillText("Waiting for Claude Code activity...", w / 2, h / 2 + 4);

  ctx.fillStyle = "rgba(120, 120, 150, 0.5)";
  ctx.font = "9px -apple-system, sans-serif";
  ctx.fillText("Watching ~/.claude/projects/", w / 2, h / 2 + 20);
}

function drawFloor(
  ctx: CanvasRenderingContext2D,
  state: OfficeState,
  tileSize: number,
  scale: number
): void {
  for (let row = 0; row < state.grid_rows; row++) {
    for (let col = 0; col < state.grid_cols; col++) {
      const x = col * tileSize;
      const y = row * tileSize;

      // Base tile
      const isLight = (row + col) % 2 === 0;
      ctx.fillStyle = isLight ? "#3D3D5C" : "#383856";
      ctx.fillRect(x, y, tileSize, tileSize);

      // Subtle tile border (grout lines)
      ctx.fillStyle = "rgba(0,0,0,0.08)";
      ctx.fillRect(x, y, tileSize, 0.5);
      ctx.fillRect(x, y, 0.5, tileSize);

      // Tile highlight (top-left edge catch)
      ctx.fillStyle = "rgba(255,255,255,0.02)";
      ctx.fillRect(x + 0.5, y + 0.5, tileSize - 1, 1);
    }
  }

  // Baseboard along bottom wall
  const baseY = state.grid_rows * tileSize - 3 * scale;
  ctx.fillStyle = "#2A2A42";
  ctx.fillRect(0, baseY, state.grid_cols * tileSize, 3 * scale);
  ctx.fillStyle = "rgba(255,255,255,0.03)";
  ctx.fillRect(0, baseY, state.grid_cols * tileSize, 0.5 * scale);
}

function drawWalls(
  ctx: CanvasRenderingContext2D,
  state: OfficeState,
  tileSize: number,
  scale: number
): void {
  const wallThickness = 5 * scale;
  const gridW = state.grid_cols * tileSize;
  const gridH = state.grid_rows * tileSize;
  const doorCol = state.door_pos.col;

  // Wall color with gradient
  const wallGrad = ctx.createLinearGradient(0, -wallThickness, 0, 0);
  wallGrad.addColorStop(0, "#3a3a58");
  wallGrad.addColorStop(1, "#2a2a44");

  // Top wall (with door gap)
  ctx.fillStyle = wallGrad;
  ctx.fillRect(0, -wallThickness, doorCol * tileSize, wallThickness);
  ctx.fillRect((doorCol + 1) * tileSize, -wallThickness, (state.grid_cols - doorCol - 1) * tileSize, wallThickness);

  // Wall top highlight
  ctx.fillStyle = "rgba(255,255,255,0.06)";
  ctx.fillRect(0, -wallThickness, doorCol * tileSize, scale);
  ctx.fillRect((doorCol + 1) * tileSize, -wallThickness, (state.grid_cols - doorCol - 1) * tileSize, scale);

  // Side walls
  const sideGrad = ctx.createLinearGradient(-wallThickness, 0, 0, 0);
  sideGrad.addColorStop(0, "#2a2a44");
  sideGrad.addColorStop(1, "#33334f");
  ctx.fillStyle = sideGrad;
  ctx.fillRect(-wallThickness, -wallThickness, wallThickness, gridH + wallThickness * 2);

  const rightGrad = ctx.createLinearGradient(gridW, 0, gridW + wallThickness, 0);
  rightGrad.addColorStop(0, "#33334f");
  rightGrad.addColorStop(1, "#2a2a44");
  ctx.fillStyle = rightGrad;
  ctx.fillRect(gridW, -wallThickness, wallThickness, gridH + wallThickness * 2);

  // Bottom wall
  ctx.fillStyle = "#2a2a44";
  ctx.fillRect(-wallThickness, gridH, gridW + wallThickness * 2, wallThickness);

  // Window on left wall
  drawWallWindow(ctx, -wallThickness * 0.3, gridH * 0.3, wallThickness * 0.6, gridH * 0.25, scale);

  // Window on right wall
  drawWallWindow(ctx, gridW + wallThickness * 0.1, gridH * 0.3, wallThickness * 0.6, gridH * 0.25, scale);
}

function drawWallWindow(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  _scale: number
): void {
  // Window frame
  ctx.fillStyle = "#444460";
  ctx.fillRect(x - 1, y - 1, w + 2, h + 2);

  // Sky gradient
  const skyGrad = ctx.createLinearGradient(x, y, x, y + h);
  skyGrad.addColorStop(0, "#1a2a4a");
  skyGrad.addColorStop(0.6, "#2a3a5a");
  skyGrad.addColorStop(1, "#3a4a6a");
  ctx.fillStyle = skyGrad;
  ctx.fillRect(x, y, w, h);

  // Stars
  ctx.fillStyle = "rgba(255,255,255,0.4)";
  ctx.fillRect(x + w * 0.2, y + h * 0.2, 1, 1);
  ctx.fillRect(x + w * 0.7, y + h * 0.4, 1, 1);
  ctx.fillRect(x + w * 0.4, y + h * 0.15, 1, 1);

  // Window divider
  ctx.fillStyle = "#444460";
  ctx.fillRect(x + w / 2 - 0.5, y, 1, h);
  ctx.fillRect(x, y + h / 2 - 0.5, w, 1);
}

function drawAgent(
  ctx: CanvasRenderingContext2D,
  agent: AgentCharacter,
  scale: number
): void {
  const sprites = getSprites(agent.character_index);
  const stateName = agent.state.state;
  const frame = agent.anim_frame % 4;
  const key = `${stateName}_${agent.direction}_${frame}`;

  let sprite = sprites.get(key);
  // Fallback: try using a base state sprite if the new state sprite is missing
  if (!sprite) {
    const fallbackState = getFallbackState(stateName);
    const fallbackKey = `${fallbackState}_${agent.direction}_${frame}`;
    sprite = sprites.get(fallbackKey);
  }
  if (!sprite) return;

  const charScale = scale * 1.1;

  // Offset seated characters down so they sit below/in front of their desk, not on top of it
  const isSeated = stateName === "Idle" || stateName === "Typing" || stateName === "Reading" || stateName === "Fidgeting";
  // Thinking and Celebrating happen at the desk but the character stands
  const isStanding = stateName === "Thinking" || stateName === "Celebrating" || stateName === "Waving";
  const seatOffset = (isSeated && !isStanding) ? TILE_SIZE * scale * 0.75 : 0;

  const ax = agent.x * scale - (CHAR_SIZE * charScale) / 2;
  const ay = agent.y * scale - (CHAR_SIZE * charScale) + seatOffset;

  ctx.drawImage(sprite, ax, ay, CHAR_SIZE * charScale, CHAR_SIZE * charScale);

  // Speech bubble during meetings
  if (stateName === "Meeting") {
    drawSpeechBubble(ctx, agent.x * scale, ay, scale, animTime);
  }

  // Thought bubble during Thinking
  if (stateName === "Thinking") {
    drawThoughtBubble(ctx, agent.x * scale, ay, scale, animTime);
  }

  // Speech bubble during Whiteboard sessions
  if (stateName === "Whiteboard") {
    drawSpeechBubble(ctx, agent.x * scale, ay, scale, animTime);
  }
}

/** Get a fallback state name for sprite lookup when the exact state sprite is not found */
function getFallbackState(stateName: string): string {
  switch (stateName) {
    case "Fidgeting":
      return "Idle";
    case "WaterCooler":
      return "Walking";
    case "Whiteboard":
      return "Walking";
    case "Waving":
      return "Idle";
    case "Thinking":
      return "Idle";
    case "Celebrating":
      return "Idle";
    case "GettingCoffee":
      return "Walking";
    default:
      return "Idle";
  }
}

function drawLighting(
  ctx: CanvasRenderingContext2D,
  state: OfficeState,
  tileSize: number
): void {
  const gridW = state.grid_cols * tileSize;
  const gridH = state.grid_rows * tileSize;
  const cx = gridW / 2;
  const cy = gridH / 2;

  // Subtle radial vignette (darker edges)
  const radius = Math.max(gridW, gridH) * 0.7;
  const vignette = ctx.createRadialGradient(cx, cy, radius * 0.3, cx, cy, radius);
  vignette.addColorStop(0, "rgba(0,0,0,0)");
  vignette.addColorStop(1, "rgba(0,0,0,0.15)");
  ctx.fillStyle = vignette;
  ctx.fillRect(0, 0, gridW, gridH);

  // Overhead light pool (warm center glow)
  const light = ctx.createRadialGradient(cx, cy * 0.6, 0, cx, cy * 0.6, radius * 0.5);
  light.addColorStop(0, "rgba(255, 240, 200, 0.03)");
  light.addColorStop(1, "rgba(255, 240, 200, 0)");
  ctx.fillStyle = light;
  ctx.fillRect(0, 0, gridW, gridH);
}

function drawStatusBar(
  ctx: CanvasRenderingContext2D,
  state: OfficeState,
  width: number,
  height: number
): void {
  // Bar background (use cached gradient)
  const grads = getCachedGradients(ctx, width, height);
  ctx.fillStyle = grads.barGrad;
  ctx.fillRect(0, height - 20, width, 20);

  // Separator line
  ctx.fillStyle = "rgba(255,255,255,0.05)";
  ctx.fillRect(0, height - 20, width, 0.5);

  ctx.fillStyle = "#999";
  ctx.font = "10px -apple-system, 'SF Pro Text', sans-serif";
  ctx.textAlign = "left";

  const agentCount = state.agents.length;
  const deskCount = state.desks.length;

  // Active indicator dot
  if (agentCount > 0) {
    ctx.fillStyle = "#4ADE80";
    ctx.beginPath();
    ctx.arc(10, height - 10, 3, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = "#999";
    ctx.fillText(`${agentCount} agent${agentCount !== 1 ? "s" : ""}`, 18, height - 6);
  } else {
    ctx.fillStyle = "#555";
    ctx.beginPath();
    ctx.arc(10, height - 10, 3, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = "#666";
    ctx.fillText("idle", 18, height - 6);
  }

  // Desk count on right
  ctx.textAlign = "right";
  ctx.fillStyle = "#777";
  ctx.fillText(`${deskCount} desk${deskCount !== 1 ? "s" : ""}`, width - 8, height - 6);
}
