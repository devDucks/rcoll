# rcoll — Telescope Collimation Overlay Tool

A transparent, always-on-top floating overlay for collimating Newtonian telescopes.
Drag it over your camera live view, place concentric collimation circles, nudge until
everything lines up.

---

## Architecture Overview

**Language:** Rust  
**GUI framework:** `egui` + `eframe` (supports transparency, always-on-top, cross-platform)  
**Future image analysis:** `image` crate + optional OpenCV bindings or pure-Rust circle detection  

**Window model:**
- Single transparent frameless window, always-on-top
- Click-through mode (optional): mouse events pass through to the window behind
- Draggable via a thin grip area or hold modifier key

---

## Phase 1 — Project Bootstrap

- [ ] Initialize Rust project with `cargo init`
- [ ] Add dependencies to `Cargo.toml`:
  - `eframe` (egui backend) with transparency feature flags
  - `egui` for UI primitives
  - `serde` + `serde_json` for saving/loading circle configurations
- [ ] Configure `eframe` window options:
  - `transparent: true`
  - `decorations: false` (frameless)
  - `always_on_top: true`
  - `resizable: true`
  - Initial size: 600×600, centered
- [ ] Verify blank transparent window launches on Linux, macOS, Windows

---

## Phase 2 — Overlay Window UX

- [ ] Implement window dragging:
  - Detect drag on any empty (non-circle) area
  - Use `egui` pointer delta to move window position
- [ ] Implement window resizing (corner handles or resize grip)
- [ ] Global opacity slider (0 %–90 %) stored in app state
  - Affects background fill alpha; circles remain fully visible
- [ ] Minimal HUD toolbar (collapsible strip at top or side):
  - Add circle button
  - Toggle click-through mode
  - Reset / clear all circles
  - Save / Load layout
  - Opacity slider
- [ ] Keyboard shortcut: `H` hides/shows the HUD without closing the app
- [ ] Keyboard shortcut: `Esc` quits

---

## Phase 3 — Collimation Circles

Each circle has:
- `center: (f32, f32)` — position in window-local coords
- `radius: f32`
- `color: egui::Color32`
- `label: String` (e.g. "Primary", "Secondary", "Center dot")
- `stroke_width: f32`

### 3a — Circle Placement

- [ ] Click "Add circle" → next click on the overlay canvas places a new circle
      centered at the click point with a default radius
- [ ] Preset buttons for common Newtonian elements:
  - **Primary mirror** — large circle, white, thick stroke
  - **Secondary mirror** — medium circle, yellow, thick stroke
  - **Center dot** — tiny filled circle or crosshair, red
- [ ] Circle list panel (side drawer or HUD section):
  - Shows each circle with label, color swatch, visibility toggle, delete button

### 3b — Mouse Interaction

- [ ] Click inside a circle's stroke region → select it (highlight with dashed border)
- [ ] Drag selected circle → move it
- [ ] Drag circle edge (within ±8 px of circumference) → resize radius
- [ ] Right-click circle → context menu: rename, change color, duplicate, delete

### 3c — Micro-Adjustment Controls

Selected circle exposes fine controls (in HUD or floating inspector panel):

- [ ] **Position nudge**: arrow buttons move center by 0.5 px per click
- [ ] **Radius nudge**: `+` / `−` buttons adjust radius by 0.5 px per click
- [ ] **Keyboard nudge** (when circle selected):
  - Arrow keys → move center ±1 px
  - `[` / `]` → decrease / increase radius by 1 px
  - Hold `Shift` → 0.1 px precision for all nudges
  - Hold `Ctrl` → 5 px coarse steps
- [ ] Numeric input fields for exact center X, Y, and radius values

---

## Phase 4 — Persistence

- [ ] Define `Config` struct (serde):
  - List of circles with all fields
  - Window size, position, opacity
- [ ] Auto-save config to `~/.config/rcoll/layout.json` on change
- [ ] Load config on startup; fall back to defaults if missing
- [ ] CLI flag `--config <path>` to use a custom config file
- [ ] "Save as preset" / "Load preset" from named JSON files

---

## Phase 5 — Polish & Usability

- [ ] Concentric alignment guide: draw faint radial lines from the common center
      to help judge concentricity visually
- [ ] Centering helper: button that computes the geometric mean center of all
      circles and snaps them all to share that center
- [ ] Grid / crosshair overlay toggle (Rule of thirds or +/× crosshair)
- [ ] Zoom-in loupe: magnified inset view of the intersection area (egui texture)
- [ ] Color themes: dark, light, red-light (for night-vision preservation)
- [ ] Responsive HUD that collapses to an icon strip when window is small

---

## Phase 6 — Auto-Detection (Image Analysis)

Allows rcoll to grab a screenshot of the region beneath the overlay window and
attempt to detect circles automatically using a Hough Circle Transform.

### 6a — Screen Capture

- [ ] Add dependency: `xcap` or `screenshots` crate for cross-platform screen capture
- [ ] Capture the exact rectangle that the rcoll window covers (minus the HUD strip)
- [ ] Convert captured image to `image::DynamicImage`

### 6b — Circle Detection

- [ ] Add dependency: `imageproc` crate (provides Hough circle transform)
- [ ] Pre-process captured image:
  - Convert to grayscale
  - Apply Gaussian blur to reduce noise
  - Canny edge detection
- [ ] Run Hough Circle Transform with configurable parameters:
  - `min_radius`, `max_radius` (derived from window size)
  - `min_distance` between circle centers
  - Accumulator threshold
- [ ] Return top N candidate circles sorted by accumulator score
- [ ] Map detected circle positions back to window-local coordinates

### 6c — Auto-Detection UX

- [ ] "Auto-detect circles" button in HUD triggers capture + detection
- [ ] Show detected candidates as faint ghost circles
- [ ] User clicks to accept or dismiss each ghost circle
- [ ] Accepted circle becomes a normal collimation circle
- [ ] Detection parameters panel (expandable) with sliders for threshold, radius range
- [ ] "Re-detect" button to re-run with adjusted parameters

---

## Phase 7 — Packaging & Distribution

- [ ] Write `README.md` with screenshots, installation instructions, usage guide
- [ ] Add `Makefile` / `justfile` with common tasks: `build`, `run`, `release`, `install`
- [ ] Cross-compile targets: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`
- [ ] GitHub Actions CI: build + test on all three platforms
- [ ] GitHub Actions release: publish binaries on tag push
- [ ] Linux: generate `.deb` and `.rpm` packages via `cargo-deb` / `cargo-generate-rpm`
- [ ] macOS: bundle as `.app` via `cargo-bundle`
- [ ] Windows: create installer via `cargo-wix` (WiX toolset)

---

## Dependency Reference

| Crate | Purpose |
|---|---|
| `eframe` | Window management + egui backend |
| `egui` | Immediate-mode GUI |
| `serde` + `serde_json` | Config serialization |
| `dirs` | Platform config directory (`~/.config`) |
| `xcap` or `screenshots` | Screen capture (Phase 6) |
| `image` | Image loading/conversion (Phase 6) |
| `imageproc` | Hough transform, edge detection (Phase 6) |

---

## Milestone Summary

| Milestone | Phases | Deliverable |
|---|---|---|
| M1 — Proof of concept | 1–2 | Transparent draggable window |
| M2 — Core tool | 3–4 | Full circle placement + persistence |
| M3 — Polished release | 5 | v1.0 ready for end users |
| M4 — Smart detection | 6 | Auto-detect mirrors from camera feed |
| M5 — Distribution | 7 | Packaged binaries for all platforms |
