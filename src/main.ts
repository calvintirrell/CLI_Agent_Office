import { listen } from "@tauri-apps/api/event";
import type { OfficeState } from "./types";
import { render } from "./renderer";

let currentState: OfficeState | null = null;
let lastRenderedGeneration: number = -1;

/** Initialize the canvas and start the render loop */
function init(): void {
  const canvas = document.getElementById("office-canvas") as HTMLCanvasElement;
  if (!canvas) {
    console.error("Canvas element not found");
    return;
  }

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    console.error("Could not get 2D context");
    return;
  }

  // Listen for office state updates from the Rust backend
  const unlistenPromise = listen<OfficeState>("office_state", (event) => {
    currentState = event.payload;
  });

  // Store unlisten for cleanup on page unload
  window.addEventListener("beforeunload", () => {
    unlistenPromise.then((unlisten) => unlisten());
  });

  // Render loop at 60fps (display framerate), state updates come at ~30fps from backend
  function frame(): void {
    // Skip render if the backend state generation hasn't changed
    const currentGen = currentState?.generation ?? -1;
    if (currentGen !== lastRenderedGeneration) {
      render(ctx!, canvas, currentState);
      lastRenderedGeneration = currentGen;
    }
    requestAnimationFrame(frame);
  }

  requestAnimationFrame(frame);
}

// Start when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
