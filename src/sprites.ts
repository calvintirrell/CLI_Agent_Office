/**
 * Polished procedural pixel-art sprite generator.
 * Diverse characters (gender, skin tone, hair), office chairs, furniture.
 */

const CHAR_SIZE = 24;

/** Character identity — gender and appearance traits */
interface CharacterIdentity {
  gender: "male" | "female";
  skin: string;
  skinShadow: string;
  hair: string;
  hairHighlight: string;
  shirt: string;
  shirtShadow: string;
  pants: string;
  pantsShadow: string;
  shoes: string;
  outline: string;
}

/** 6 diverse characters — mixed gender, skin tones, and style */
const CHARACTERS: CharacterIdentity[] = [
  // 0: Male, light brown skin, dark hair, blue shirt
  { gender: "male",
    skin: "#D4A574", skinShadow: "#B88B5E", hair: "#1A1A1A", hairHighlight: "#333333",
    shirt: "#2980B9", shirtShadow: "#1F6DA0", pants: "#2C3E50", pantsShadow: "#1E2D3D",
    shoes: "#1A1A1A", outline: "#1A1A2E" },
  // 1: Female, dark brown skin, dark curly hair, red blouse
  { gender: "female",
    skin: "#8D5524", skinShadow: "#704218", hair: "#0D0D0D", hairHighlight: "#2A2A2A",
    shirt: "#C0392B", shirtShadow: "#A02E23", pants: "#2C3E50", pantsShadow: "#1E2D3D",
    shoes: "#2C1A0A", outline: "#0F0F1E" },
  // 2: Male, fair/light skin, brown hair, green shirt
  { gender: "male",
    skin: "#FFDBB4", skinShadow: "#E8C49A", hair: "#6B4226", hairHighlight: "#8B5A3A",
    shirt: "#27AE60", shirtShadow: "#1E8C4D", pants: "#34495E", pantsShadow: "#263747",
    shoes: "#1A1A1A", outline: "#1A1A2E" },
  // 3: Female, medium/olive skin, auburn hair, purple blouse
  { gender: "female",
    skin: "#C68642", skinShadow: "#A86E34", hair: "#922B21", hairHighlight: "#B03A2E",
    shirt: "#8E44AD", shirtShadow: "#723690", pants: "#2C3E50", pantsShadow: "#1E2D3D",
    shoes: "#1A1A1A", outline: "#1A1A2E" },
  // 4: Male, deep brown skin, black short hair, orange shirt
  { gender: "male",
    skin: "#6F4E37", skinShadow: "#5A3D2B", hair: "#0A0A0A", hairHighlight: "#1A1A1A",
    shirt: "#E67E22", shirtShadow: "#CA6B1B", pants: "#34495E", pantsShadow: "#263747",
    shoes: "#111111", outline: "#0A0A18" },
  // 5: Female, light/peach skin, blonde hair, teal blouse
  { gender: "female",
    skin: "#FFE0BD", skinShadow: "#F0CCA6", hair: "#D4A017", hairHighlight: "#E8B830",
    shirt: "#16A085", shirtShadow: "#11806A", pants: "#2C3E50", pantsShadow: "#1E2D3D",
    shoes: "#2C1A0A", outline: "#1A1A2E" },
];

function px(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, color: string) {
  ctx.fillStyle = color;
  ctx.fillRect(Math.round(x), Math.round(y), w, h);
}

/** Draw an office rolling chair */
function drawChair(ctx: CanvasRenderingContext2D, yOff: number) {
  // Seat cushion
  px(ctx, 5, 14 + yOff, 14, 3, "#3A3A3A");
  px(ctx, 5, 14 + yOff, 14, 1, "#4A4A4A"); // top highlight
  // Seat sides (arm rests)
  px(ctx, 4, 13 + yOff, 1, 4, "#333");
  px(ctx, 19, 13 + yOff, 1, 4, "#333");
  // Chair back
  px(ctx, 6, 10 + yOff, 12, 4, "#2E2E2E");
  px(ctx, 6, 10 + yOff, 12, 1, "#3E3E3E"); // top highlight
  px(ctx, 7, 11 + yOff, 10, 2, "#333"); // back padding

  // Chair stem
  px(ctx, 11, 17 + yOff, 2, 3, "#555");

  // Chair base (star shape - simplified as cross)
  px(ctx, 7, 20 + yOff, 10, 1, "#444");
  px(ctx, 11, 19 + yOff, 2, 2, "#444");

  // Wheels (small circles)
  ctx.fillStyle = "#333";
  ctx.beginPath();
  ctx.arc(8, 21 + yOff, 1.2, 0, Math.PI * 2);
  ctx.fill();
  ctx.beginPath();
  ctx.arc(16, 21 + yOff, 1.2, 0, Math.PI * 2);
  ctx.fill();
  ctx.beginPath();
  ctx.arc(12, 21.5 + yOff, 1.2, 0, Math.PI * 2);
  ctx.fill();
}

/** Draw a character — seated states show chair, walking states don't */
function drawCharacter(
  ctx: CanvasRenderingContext2D,
  c: CharacterIdentity,
  direction: number,
  frame: number,
  state: string
): void {
  ctx.clearRect(0, 0, CHAR_SIZE, CHAR_SIZE);

  const isSeated = state === "Idle" || state === "Typing" || state === "Reading" || state === "Fidgeting";
  const isMoving = state === "Walking" || state === "Entering" || state === "Leaving" || state === "Meeting"
    || (state === "WaterCooler") || (state === "GettingCoffee") || (state === "Whiteboard");
  const isStanding = state === "Waving" || state === "Thinking" || state === "Celebrating";

  const bobY = (state === "Idle" || state === "Fidgeting") ? Math.sin(frame * 0.4) * 0.4 : 0;
  const walkBob = isMoving ? Math.sin(frame * Math.PI) * 1 : 0;
  const celebrateBob = state === "Celebrating" ? Math.sin(frame * Math.PI * 2) * 2 : 0;
  const yOff = Math.round(bobY + walkBob + celebrateBob);

  // Ground shadow
  if (!isSeated || isStanding) {
    ctx.fillStyle = "rgba(0,0,0,0.15)";
    ctx.beginPath();
    ctx.ellipse(12, 22, 5, 2, 0, 0, Math.PI * 2);
    ctx.fill();
  }

  // Draw chair for seated states (but not for standing-at-desk states like Thinking/Celebrating)
  if (isSeated && !isStanding) {
    drawChair(ctx, yOff);
  }

  // Seated characters are shifted up slightly to sit IN the chair
  const seatOff = (isSeated && !isStanding) ? -2 : 0;

  // === HAIR (back layer for female long hair from behind) ===
  if (direction === 1 && c.gender === "female") {
    // Long hair from behind
    px(ctx, 7, 1 + yOff + seatOff, 10, 8, c.hair);
    px(ctx, 8, 9 + yOff + seatOff, 8, 2, c.hair); // hair draping down back
  }

  // === HEAD ===
  if (direction === 1) {
    // Back of head
    px(ctx, 8, 1 + yOff + seatOff, 8, 5, c.hair);
    px(ctx, 7, 2 + yOff + seatOff, 1, 3, c.hair);
    px(ctx, 16, 2 + yOff + seatOff, 1, 3, c.hair);
    px(ctx, 10, 6 + yOff + seatOff, 4, 1, c.skinShadow);
  } else {
    // Hair top
    px(ctx, 8, 0 + yOff + seatOff, 8, 3, c.hair);
    px(ctx, 10, 0 + yOff + seatOff, 3, 1, c.hairHighlight);

    if (c.gender === "female") {
      // Long hair sides
      px(ctx, 6, 1 + yOff + seatOff, 2, 7, c.hair);
      px(ctx, 16, 1 + yOff + seatOff, 2, 7, c.hair);
      // Hair drape
      px(ctx, 6, 8 + yOff + seatOff, 2, 2, c.hair);
      px(ctx, 16, 8 + yOff + seatOff, 2, 2, c.hair);
      px(ctx, 7, 1 + yOff + seatOff, 1, 2, c.hairHighlight);
    } else {
      // Short hair sides
      px(ctx, 7, 1 + yOff + seatOff, 1, 3, c.hair);
      px(ctx, 16, 1 + yOff + seatOff, 1, 3, c.hair);
    }

    // Face
    px(ctx, 8, 3 + yOff + seatOff, 8, 4, c.skin);
    // Face shadow
    if (direction === 2) {
      px(ctx, 8, 3 + yOff + seatOff, 1, 4, c.skinShadow);
    } else if (direction === 3) {
      px(ctx, 15, 3 + yOff + seatOff, 1, 4, c.skinShadow);
    } else {
      px(ctx, 8, 5 + yOff + seatOff, 1, 2, c.skinShadow);
      px(ctx, 15, 5 + yOff + seatOff, 1, 2, c.skinShadow);
    }

    // Eyes
    if (direction === 0) {
      px(ctx, 9, 4 + yOff + seatOff, 2, 2, "#FFF");
      px(ctx, 13, 4 + yOff + seatOff, 2, 2, "#FFF");
      px(ctx, 10, 4 + yOff + seatOff, 1, 2, "#222");
      px(ctx, 14, 4 + yOff + seatOff, 1, 2, "#222");
      px(ctx, 11, 6 + yOff + seatOff, 2, 1, c.skinShadow);
    } else if (direction === 2) {
      px(ctx, 13, 4 + yOff + seatOff, 2, 2, "#FFF");
      px(ctx, 14, 4 + yOff + seatOff, 1, 2, "#222");
      px(ctx, 14, 6 + yOff + seatOff, 1, 1, c.skinShadow);
    } else if (direction === 3) {
      px(ctx, 9, 4 + yOff + seatOff, 2, 2, "#FFF");
      px(ctx, 9, 4 + yOff + seatOff, 1, 2, "#222");
      px(ctx, 9, 6 + yOff + seatOff, 1, 1, c.skinShadow);
    }

    px(ctx, 10, 7 + yOff + seatOff, 4, 1, c.skinShadow); // neck
  }

  // === TORSO ===
  px(ctx, 6, 8 + yOff + seatOff, 12, 5, c.shirt);
  px(ctx, 6, 11 + yOff + seatOff, 12, 2, c.shirtShadow);
  px(ctx, 10, 8 + yOff + seatOff, 4, 1, c.skin); // collar

  // === ARMS ===
  if (state === "Typing") {
    const armBob = frame % 2 === 0 ? 0 : 1;
    px(ctx, 4, 9 + yOff + seatOff, 2, 3, c.shirt);
    px(ctx, 18, 9 + yOff + seatOff, 2, 3, c.shirt);
    px(ctx, 4, 12 + yOff + seatOff + armBob, 2, 1, c.skin);
    px(ctx, 18, 12 + yOff + seatOff - armBob, 2, 1, c.skin);
  } else if (state === "Reading") {
    px(ctx, 4, 8 + yOff + seatOff, 2, 4, c.shirt);
    px(ctx, 18, 8 + yOff + seatOff, 2, 4, c.shirt);
    px(ctx, 4, 8 + yOff + seatOff, 2, 1, c.skin);
    px(ctx, 18, 8 + yOff + seatOff, 2, 1, c.skin);
  } else if (state === "Waving") {
    // Right arm raised high (waving)
    const waveBob = frame % 2 === 0 ? -1 : 1;
    px(ctx, 4, 9 + yOff + seatOff, 2, 4, c.shirt); // left arm normal
    px(ctx, 4, 13 + yOff + seatOff, 2, 1, c.skin);
    px(ctx, 18, 4 + yOff + seatOff + waveBob, 2, 4, c.shirt); // right arm raised
    px(ctx, 18, 4 + yOff + seatOff + waveBob, 2, 1, c.skin); // hand
  } else if (state === "Thinking") {
    // Hand on chin pose
    px(ctx, 4, 9 + yOff + seatOff, 2, 4, c.shirt); // left arm normal
    px(ctx, 4, 13 + yOff + seatOff, 2, 1, c.skin);
    px(ctx, 18, 7 + yOff + seatOff, 2, 3, c.shirt); // right arm up to chin
    px(ctx, 17, 6 + yOff + seatOff, 2, 1, c.skin); // hand on chin
  } else if (state === "Celebrating") {
    // Both arms raised high (jumping celebration)
    const raiseBob = frame % 2 === 0 ? -1 : 0;
    px(ctx, 2, 3 + yOff + seatOff + raiseBob, 2, 5, c.shirt); // left arm up
    px(ctx, 2, 3 + yOff + seatOff + raiseBob, 2, 1, c.skin); // left hand
    px(ctx, 20, 3 + yOff + seatOff + raiseBob, 2, 5, c.shirt); // right arm up
    px(ctx, 20, 3 + yOff + seatOff + raiseBob, 2, 1, c.skin); // right hand
  } else if (state === "Fidgeting") {
    // Stretch animation: arms out to sides
    const stretchPhase = (frame % 4);
    if (stretchPhase < 2) {
      // Stretching out
      px(ctx, 2, 9 + yOff + seatOff, 4, 2, c.shirt);
      px(ctx, 18, 9 + yOff + seatOff, 4, 2, c.shirt);
      px(ctx, 2, 9 + yOff + seatOff, 2, 1, c.skin);
      px(ctx, 20, 9 + yOff + seatOff, 2, 1, c.skin);
    } else {
      // Arms back in, looking around
      px(ctx, 4, 9 + yOff + seatOff, 2, 4, c.shirt);
      px(ctx, 18, 9 + yOff + seatOff, 2, 4, c.shirt);
      px(ctx, 4, 13 + yOff + seatOff, 2, 1, c.skin);
      px(ctx, 18, 13 + yOff + seatOff, 2, 1, c.skin);
    }
  } else if (state === "GettingCoffee") {
    // Right arm holding a cup out front
    px(ctx, 4, 9 + yOff + seatOff, 2, 4, c.shirt); // left arm
    px(ctx, 4, 13 + yOff + seatOff, 2, 1, c.skin);
    px(ctx, 18, 9 + yOff + seatOff, 2, 3, c.shirt); // right arm bent
    px(ctx, 18, 9 + yOff + seatOff, 2, 1, c.skin); // hand
    // Small cup
    px(ctx, 19, 8 + yOff + seatOff, 2, 2, "#DDD");
    px(ctx, 19, 8.5 + yOff + seatOff, 2, 1, "#8B4513");
  } else if (state === "Whiteboard") {
    // One arm pointing forward (at whiteboard)
    px(ctx, 4, 9 + yOff + seatOff, 2, 4, c.shirt); // left arm
    px(ctx, 4, 13 + yOff + seatOff, 2, 1, c.skin);
    const pointBob = frame % 2 === 0 ? 0 : 1;
    px(ctx, 18, 7 + yOff + seatOff + pointBob, 2, 4, c.shirt); // right arm raised pointing
    px(ctx, 19, 7 + yOff + seatOff + pointBob, 2, 1, c.skin); // pointing hand
  } else {
    const swing = isMoving ? (frame % 2 === 0 ? 1 : -1) : 0;
    px(ctx, 4, 9 + yOff + seatOff + swing, 2, 4, c.shirt);
    px(ctx, 18, 9 + yOff + seatOff - swing, 2, 4, c.shirt);
    px(ctx, 4, 9 + yOff + seatOff + swing, 2, 1, c.shirtShadow);
    px(ctx, 18, 9 + yOff + seatOff - swing, 2, 1, c.shirtShadow);
    px(ctx, 4, 13 + yOff + seatOff + swing, 2, 1, c.skin);
    px(ctx, 18, 13 + yOff + seatOff - swing, 2, 1, c.skin);
  }

  // === LEGS ===
  if (isSeated && !isStanding) {
    // Legs bent forward on chair
    px(ctx, 7, 13 + yOff + seatOff, 3, 3, c.pants);
    px(ctx, 14, 13 + yOff + seatOff, 3, 3, c.pants);
    px(ctx, 9, 13 + yOff + seatOff, 1, 3, c.pantsShadow);
    px(ctx, 14, 13 + yOff + seatOff, 1, 3, c.pantsShadow);
    // Lower legs hanging
    px(ctx, 7, 16 + yOff + seatOff, 3, 2, c.pants);
    px(ctx, 14, 16 + yOff + seatOff, 3, 2, c.pants);
    // Shoes
    px(ctx, 7, 18 + yOff + seatOff, 3, 1, c.shoes);
    px(ctx, 14, 18 + yOff + seatOff, 3, 1, c.shoes);
  } else {
    const legSpread = isMoving ? (frame % 2 === 0 ? 2 : -1) : 0;
    px(ctx, 7 + legSpread, 13 + yOff + seatOff, 3, 5, c.pants);
    px(ctx, 14 - legSpread, 13 + yOff + seatOff, 3, 5, c.pants);
    px(ctx, 9 + legSpread, 13 + yOff + seatOff, 1, 5, c.pantsShadow);
    px(ctx, 14 - legSpread, 13 + yOff + seatOff, 1, 5, c.pantsShadow);
    px(ctx, 6 + legSpread, 18 + yOff + seatOff, 4, 2, c.shoes);
    px(ctx, 14 - legSpread, 18 + yOff + seatOff, 4, 2, c.shoes);
    px(ctx, 7 + legSpread, 18 + yOff + seatOff, 2, 1, "#333");
    px(ctx, 15 - legSpread, 18 + yOff + seatOff, 2, 1, "#333");
  }
}

/** Generate all sprite frames for a character index */
export function generateCharacterSprites(charIndex: number): Map<string, HTMLCanvasElement> {
  const identity = CHARACTERS[charIndex % CHARACTERS.length];
  const sprites = new Map<string, HTMLCanvasElement>();

  const states = [
    "Idle", "Walking", "Entering", "Typing", "Reading", "Meeting", "Leaving",
    "Fidgeting", "WaterCooler", "Whiteboard", "Waving", "Thinking", "Celebrating", "GettingCoffee",
  ];
  const directions = [0, 1, 2, 3];
  const frames = [0, 1, 2, 3];

  for (const state of states) {
    for (const dir of directions) {
      for (const frame of frames) {
        const key = `${state}_${dir}_${frame}`;
        const canvas = document.createElement("canvas");
        canvas.width = CHAR_SIZE;
        canvas.height = CHAR_SIZE;
        const ctx = canvas.getContext("2d")!;

        if (dir === 3) {
          drawCharacter(ctx, identity, 2, frame, state);
          const flipped = document.createElement("canvas");
          flipped.width = CHAR_SIZE;
          flipped.height = CHAR_SIZE;
          const fCtx = flipped.getContext("2d")!;
          fCtx.translate(CHAR_SIZE, 0);
          fCtx.scale(-1, 1);
          fCtx.drawImage(canvas, 0, 0);
          sprites.set(key, flipped);
        } else {
          drawCharacter(ctx, identity, dir, frame, state);
          sprites.set(key, canvas);
        }
      }
    }
  }

  return sprites;
}

/** Draw a polished desk with computer */
export function drawDesk(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  scale: number,
  hasAgent: boolean
): void {
  const s = scale;

  // Desk shadow
  ctx.fillStyle = "rgba(0,0,0,0.12)";
  ctx.beginPath();
  ctx.ellipse(x + 12 * s, y + 26 * s, 14 * s, 3 * s, 0, 0, Math.PI * 2);
  ctx.fill();

  // Desk surface
  ctx.fillStyle = "#B8860B";
  ctx.fillRect(x, y + 6 * s, 24 * s, 3 * s);
  ctx.fillStyle = "#D4A635";
  ctx.fillRect(x, y + 6 * s, 24 * s, 1 * s);
  // Front face
  ctx.fillStyle = "#8B6914";
  ctx.fillRect(x, y + 9 * s, 24 * s, 10 * s);
  // Wood grain
  ctx.fillStyle = "rgba(0,0,0,0.08)";
  ctx.fillRect(x + 3 * s, y + 11 * s, 18 * s, 0.5 * s);
  ctx.fillRect(x + 2 * s, y + 14 * s, 20 * s, 0.5 * s);
  ctx.fillRect(x + 4 * s, y + 17 * s, 16 * s, 0.5 * s);

  // Legs
  ctx.fillStyle = "#6B4E0A";
  ctx.fillRect(x + 1 * s, y + 19 * s, 2 * s, 6 * s);
  ctx.fillRect(x + 21 * s, y + 19 * s, 2 * s, 6 * s);
  ctx.fillStyle = "#8B6914";
  ctx.fillRect(x + 1 * s, y + 19 * s, 1 * s, 6 * s);
  ctx.fillRect(x + 21 * s, y + 19 * s, 1 * s, 6 * s);

  // Monitor stand
  ctx.fillStyle = "#444";
  ctx.fillRect(x + 10 * s, y + 4 * s, 4 * s, 3 * s);
  ctx.fillStyle = "#555";
  ctx.fillRect(x + 8 * s, y + 5.5 * s, 8 * s, 1 * s);

  // Monitor
  ctx.fillStyle = "#2A2A2A";
  ctx.fillRect(x + 5 * s, y - 6 * s, 14 * s, 11 * s);
  ctx.fillStyle = "#1A1A1A";
  ctx.fillRect(x + 5 * s, y - 6 * s, 14 * s, 1 * s);
  ctx.fillRect(x + 5 * s, y - 6 * s, 1 * s, 11 * s);
  ctx.fillRect(x + 18 * s, y - 6 * s, 1 * s, 11 * s);
  ctx.fillRect(x + 5 * s, y + 4 * s, 14 * s, 1 * s);

  // Screen
  if (hasAgent) {
    const grad = ctx.createLinearGradient(x + 6 * s, y - 5 * s, x + 6 * s, y + 4 * s);
    grad.addColorStop(0, "#1a2634");
    grad.addColorStop(1, "#0d1b2a");
    ctx.fillStyle = grad;
    ctx.fillRect(x + 6 * s, y - 5 * s, 12 * s, 9 * s);
    const colors = ["#5CB3FF", "#82E0AA", "#F0E68C", "#5CB3FF", "#DDA0DD"];
    for (let i = 0; i < 5; i++) {
      ctx.fillStyle = colors[i];
      const lineW = (3 + Math.sin(i * 2.1) * 4) * s;
      ctx.fillRect(x + 7 * s, y + (-4 + i * 1.8) * s, lineW, 0.8 * s);
    }
    ctx.fillStyle = "rgba(90, 160, 255, 0.06)";
    ctx.fillRect(x + 2 * s, y - 8 * s, 20 * s, 6 * s);
  } else {
    ctx.fillStyle = "#111";
    ctx.fillRect(x + 6 * s, y - 5 * s, 12 * s, 9 * s);
    ctx.fillStyle = "#333";
    ctx.fillRect(x + 11.5 * s, y + 3 * s, 1 * s, 0.5 * s);
  }

  // Keyboard
  ctx.fillStyle = "#444";
  ctx.fillRect(x + 6 * s, y + 7.5 * s, 12 * s, 3 * s);
  ctx.fillStyle = "#555";
  ctx.fillRect(x + 6 * s, y + 7.5 * s, 12 * s, 0.5 * s);
  ctx.fillStyle = "#3A3A3A";
  for (let row = 0; row < 2; row++) {
    for (let col = 0; col < 6; col++) {
      ctx.fillRect(x + (6.5 + col * 2) * s, y + (8 + row * 1.2) * s, 1.5 * s, 0.8 * s);
    }
  }

  // Coffee mug
  ctx.fillStyle = "#DDD";
  ctx.fillRect(x + 20 * s, y + 3 * s, 3 * s, 3 * s);
  ctx.fillStyle = "#A52A2A";
  ctx.fillRect(x + 20 * s, y + 3.5 * s, 3 * s, 2 * s);
  if (hasAgent) {
    ctx.fillStyle = "rgba(255,255,255,0.2)";
    ctx.fillRect(x + 21 * s, y + 2 * s, 0.5 * s, 1 * s);
    ctx.fillRect(x + 21.5 * s, y + 1.5 * s, 0.5 * s, 1 * s);
  }
}

/** Draw the office door */
export function drawDoor(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  scale: number
): void {
  const s = scale;

  ctx.fillStyle = "#3D2B1F";
  ctx.fillRect(x - 1 * s, y - 1 * s, 18 * s, 26 * s);
  ctx.fillStyle = "#5D3A1A";
  ctx.fillRect(x, y, 16 * s, 24 * s);

  const doorGrad = ctx.createLinearGradient(x, y, x + 16 * s, y);
  doorGrad.addColorStop(0, "#8B6940");
  doorGrad.addColorStop(0.5, "#A0784A");
  doorGrad.addColorStop(1, "#7B5930");
  ctx.fillStyle = doorGrad;
  ctx.fillRect(x + 1.5 * s, y + 1.5 * s, 13 * s, 21 * s);

  ctx.fillStyle = "rgba(0,0,0,0.1)";
  ctx.fillRect(x + 3 * s, y + 3 * s, 10 * s, 8 * s);
  ctx.fillRect(x + 3 * s, y + 13 * s, 10 * s, 8 * s);
  ctx.fillStyle = "rgba(255,255,255,0.05)";
  ctx.fillRect(x + 3 * s, y + 3 * s, 10 * s, 0.5 * s);
  ctx.fillRect(x + 3 * s, y + 13 * s, 10 * s, 0.5 * s);

  ctx.fillStyle = "#DAA520";
  ctx.fillRect(x + 12 * s, y + 11 * s, 2 * s, 3 * s);
  ctx.fillStyle = "#F0C040";
  ctx.fillRect(x + 12 * s, y + 11 * s, 1 * s, 1 * s);

  ctx.fillStyle = "#1a2030";
  ctx.fillRect(x + 2 * s, y - 5 * s, 12 * s, 4 * s);
  ctx.fillStyle = "#2a3a5a";
  ctx.fillRect(x + 2.5 * s, y - 4.5 * s, 5 * s, 3 * s);
  ctx.fillRect(x + 8.5 * s, y - 4.5 * s, 5 * s, 3 * s);
}

/** Draw the meeting area marker */
export function drawMeetingArea(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  tileSize: number
): void {
  ctx.fillStyle = "rgba(139, 50, 20, 0.25)";
  ctx.beginPath();
  ctx.ellipse(x + tileSize / 2, y + tileSize / 2, tileSize * 0.9, tileSize * 0.65, 0, 0, Math.PI * 2);
  ctx.fill();

  ctx.strokeStyle = "rgba(180, 80, 30, 0.2)";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.ellipse(x + tileSize / 2, y + tileSize / 2, tileSize * 0.75, tileSize * 0.5, 0, 0, Math.PI * 2);
  ctx.stroke();

  ctx.fillStyle = "rgba(160, 70, 25, 0.18)";
  ctx.beginPath();
  ctx.ellipse(x + tileSize / 2, y + tileSize / 2, tileSize * 0.55, tileSize * 0.38, 0, 0, Math.PI * 2);
  ctx.fill();

  const cx = x + tileSize / 2;
  const cy = y + tileSize / 2;
  const ds = tileSize * 0.15;
  ctx.fillStyle = "rgba(200, 100, 40, 0.15)";
  ctx.beginPath();
  ctx.moveTo(cx, cy - ds);
  ctx.lineTo(cx + ds, cy);
  ctx.lineTo(cx, cy + ds);
  ctx.lineTo(cx - ds, cy);
  ctx.fill();
}

/** Draw a speech bubble with animated dots */
export function drawSpeechBubble(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  scale: number,
  time: number
): void {
  const bw = 22 * scale;
  const bh = 12 * scale;
  const bx = x - bw / 2;
  const by = y - 16 * scale;

  ctx.fillStyle = "rgba(0,0,0,0.15)";
  ctx.beginPath();
  ctx.roundRect(bx + 1 * scale, by + 1 * scale, bw, bh, 4 * scale);
  ctx.fill();

  ctx.fillStyle = "#FFF";
  ctx.beginPath();
  ctx.roundRect(bx, by, bw, bh, 4 * scale);
  ctx.fill();

  ctx.strokeStyle = "#CCC";
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  ctx.roundRect(bx, by, bw, bh, 4 * scale);
  ctx.stroke();

  ctx.fillStyle = "#FFF";
  ctx.beginPath();
  ctx.moveTo(x - 3 * scale, by + bh);
  ctx.lineTo(x, by + bh + 5 * scale);
  ctx.lineTo(x + 3 * scale, by + bh);
  ctx.fill();

  for (let i = 0; i < 3; i++) {
    const dotPhase = (time * 3 + i * 0.8) % 3;
    const dotY = by + bh / 2 - Math.sin(dotPhase * Math.PI) * 1.5 * scale;
    const alpha = 0.3 + Math.sin(dotPhase * Math.PI) * 0.5;
    ctx.fillStyle = `rgba(80, 80, 80, ${alpha})`;
    ctx.beginPath();
    ctx.arc(bx + 5 * scale + i * 5 * scale, dotY, 1.8 * scale, 0, Math.PI * 2);
    ctx.fill();
  }
}

/** Draw a water cooler */
export function drawWaterCooler(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  scale: number
): void {
  const s = scale;

  // Base/stand
  ctx.fillStyle = "#888";
  ctx.fillRect(x + 4 * s, y + 16 * s, 16 * s, 2 * s);
  ctx.fillRect(x + 10 * s, y + 18 * s, 4 * s, 6 * s);

  // Water bottle (inverted)
  ctx.fillStyle = "#6CB4EE";
  ctx.fillRect(x + 6 * s, y, 12 * s, 14 * s);
  // Bottle cap/top
  ctx.fillStyle = "#4A9AD9";
  ctx.fillRect(x + 8 * s, y - 2 * s, 8 * s, 2 * s);
  // Water highlights
  ctx.fillStyle = "rgba(255,255,255,0.2)";
  ctx.fillRect(x + 8 * s, y + 2 * s, 2 * s, 8 * s);

  // Dispenser body
  ctx.fillStyle = "#DDD";
  ctx.fillRect(x + 5 * s, y + 14 * s, 14 * s, 4 * s);
  ctx.fillStyle = "#CCC";
  ctx.fillRect(x + 5 * s, y + 14 * s, 14 * s, 1 * s);

  // Spout
  ctx.fillStyle = "#999";
  ctx.fillRect(x + 8 * s, y + 15 * s, 2 * s, 2 * s);

  // Small cup
  ctx.fillStyle = "#FFF";
  ctx.fillRect(x + 14 * s, y + 16 * s, 3 * s, 2 * s);

  // Legs
  ctx.fillStyle = "#777";
  ctx.fillRect(x + 6 * s, y + 18 * s, 2 * s, 6 * s);
  ctx.fillRect(x + 16 * s, y + 18 * s, 2 * s, 6 * s);
}

/** Draw a coffee machine */
export function drawCoffeeMachine(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  scale: number
): void {
  const s = scale;

  // Machine body
  ctx.fillStyle = "#2C2C2C";
  ctx.fillRect(x + 3 * s, y + 2 * s, 18 * s, 18 * s);
  ctx.fillStyle = "#3A3A3A";
  ctx.fillRect(x + 3 * s, y + 2 * s, 18 * s, 2 * s); // top highlight

  // Coffee display area
  ctx.fillStyle = "#1A1A1A";
  ctx.fillRect(x + 5 * s, y + 4 * s, 14 * s, 8 * s);

  // Coffee drip area
  ctx.fillStyle = "#444";
  ctx.fillRect(x + 7 * s, y + 12 * s, 10 * s, 2 * s);
  // Drip nozzle
  ctx.fillStyle = "#333";
  ctx.fillRect(x + 10 * s, y + 10 * s, 4 * s, 2 * s);

  // Cup platform
  ctx.fillStyle = "#555";
  ctx.fillRect(x + 6 * s, y + 14 * s, 12 * s, 2 * s);

  // Small coffee cup
  ctx.fillStyle = "#DDD";
  ctx.fillRect(x + 9 * s, y + 14 * s, 5 * s, 4 * s);
  ctx.fillStyle = "#8B4513";
  ctx.fillRect(x + 10 * s, y + 14 * s, 3 * s, 2 * s);

  // Steam (animated via frame, but static here)
  ctx.fillStyle = "rgba(255,255,255,0.15)";
  ctx.fillRect(x + 10 * s, y + 12 * s, 1 * s, 2 * s);
  ctx.fillRect(x + 12 * s, y + 11 * s, 1 * s, 2 * s);

  // Brand label
  ctx.fillStyle = "#C0392B";
  ctx.fillRect(x + 8 * s, y + 6 * s, 8 * s, 3 * s);
  ctx.fillStyle = "#E74C3C";
  ctx.fillRect(x + 8 * s, y + 6 * s, 8 * s, 1 * s);

  // Buttons
  ctx.fillStyle = "#4ADE80";
  ctx.fillRect(x + 16 * s, y + 16 * s, 2 * s, 2 * s);

  // Counter/base
  ctx.fillStyle = "#B8860B";
  ctx.fillRect(x + 2 * s, y + 20 * s, 20 * s, 2 * s);
  ctx.fillStyle = "#8B6914";
  ctx.fillRect(x + 2 * s, y + 22 * s, 20 * s, 2 * s);
}

/** Draw a whiteboard on the wall */
export function drawWhiteboard(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  scale: number
): void {
  const s = scale;

  // Board frame
  ctx.fillStyle = "#888";
  ctx.fillRect(x + 1 * s, y, 22 * s, 16 * s);

  // White surface
  ctx.fillStyle = "#F5F5F0";
  ctx.fillRect(x + 2 * s, y + 1 * s, 20 * s, 14 * s);

  // Some "writing" on the board
  ctx.fillStyle = "#2980B9";
  ctx.fillRect(x + 4 * s, y + 3 * s, 10 * s, 1 * s);
  ctx.fillRect(x + 4 * s, y + 5 * s, 14 * s, 1 * s);
  ctx.fillStyle = "#C0392B";
  ctx.fillRect(x + 4 * s, y + 7 * s, 8 * s, 1 * s);
  ctx.fillStyle = "#27AE60";
  ctx.fillRect(x + 4 * s, y + 9 * s, 12 * s, 1 * s);
  ctx.fillRect(x + 4 * s, y + 11 * s, 6 * s, 1 * s);

  // Tray at bottom
  ctx.fillStyle = "#777";
  ctx.fillRect(x + 3 * s, y + 15 * s, 18 * s, 2 * s);

  // Markers in tray
  ctx.fillStyle = "#E74C3C";
  ctx.fillRect(x + 5 * s, y + 15 * s, 3 * s, 1 * s);
  ctx.fillStyle = "#2980B9";
  ctx.fillRect(x + 9 * s, y + 15 * s, 3 * s, 1 * s);
  ctx.fillStyle = "#2ECC71";
  ctx.fillRect(x + 13 * s, y + 15 * s, 3 * s, 1 * s);
}

/** Draw a thought bubble (cloud-shaped with a lightbulb icon) */
export function drawThoughtBubble(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  scale: number,
  time: number
): void {
  const bw = 20 * scale;
  const bh = 14 * scale;
  const bx = x - bw / 2;
  const by = y - 20 * scale;

  // Small trailing circles (thought trail)
  ctx.fillStyle = "#FFF";
  ctx.beginPath();
  ctx.arc(x - 2 * scale, by + bh + 6 * scale, 1.5 * scale, 0, Math.PI * 2);
  ctx.fill();
  ctx.beginPath();
  ctx.arc(x - 5 * scale, by + bh + 3 * scale, 2 * scale, 0, Math.PI * 2);
  ctx.fill();

  // Shadow
  ctx.fillStyle = "rgba(0,0,0,0.12)";
  ctx.beginPath();
  ctx.ellipse(bx + bw / 2 + 1 * scale, by + bh / 2 + 1 * scale, bw / 2, bh / 2, 0, 0, Math.PI * 2);
  ctx.fill();

  // Main cloud shape
  ctx.fillStyle = "#FFF";
  ctx.beginPath();
  ctx.ellipse(bx + bw / 2, by + bh / 2, bw / 2, bh / 2, 0, 0, Math.PI * 2);
  ctx.fill();

  // Bumps on cloud
  ctx.beginPath();
  ctx.arc(bx + bw * 0.25, by + bh * 0.3, 4 * scale, 0, Math.PI * 2);
  ctx.fill();
  ctx.beginPath();
  ctx.arc(bx + bw * 0.75, by + bh * 0.3, 4 * scale, 0, Math.PI * 2);
  ctx.fill();

  ctx.strokeStyle = "#CCC";
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  ctx.ellipse(bx + bw / 2, by + bh / 2, bw / 2, bh / 2, 0, 0, Math.PI * 2);
  ctx.stroke();

  // Lightbulb icon inside
  const pulse = Math.sin(time * 4) * 0.3 + 0.7;
  ctx.fillStyle = `rgba(255, 200, 50, ${pulse})`;
  ctx.beginPath();
  ctx.arc(bx + bw / 2, by + bh / 2 - 1 * scale, 3 * scale, 0, Math.PI * 2);
  ctx.fill();
  // Bulb base
  ctx.fillStyle = "#AAA";
  ctx.fillRect(bx + bw / 2 - 1.5 * scale, by + bh / 2 + 2 * scale, 3 * scale, 2 * scale);
}

export { CHAR_SIZE };
