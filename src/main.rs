#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

// ── Circle data ───────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Circle {
    center: egui::Pos2,
    radius: f32,
    color: egui::Color32,
    label: String,
    stroke_width: f32,
    visible: bool,
}

// ── Placement mode ────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Default)]
enum PlacingMode {
    #[default]
    None,
    Generic,
    Primary,
    Secondary,
    CenterDot,
}

impl PlacingMode {
    fn is_active(&self) -> bool {
        *self != PlacingMode::None
    }

    fn default_radius(&self) -> f32 {
        match self {
            PlacingMode::Primary => 120.0,
            PlacingMode::Secondary => 60.0,
            PlacingMode::CenterDot => 6.0,
            _ => 80.0,
        }
    }

    fn default_color(&self) -> egui::Color32 {
        match self {
            PlacingMode::Secondary => egui::Color32::YELLOW,
            PlacingMode::CenterDot => egui::Color32::RED,
            _ => egui::Color32::WHITE,
        }
    }

    fn default_label(&self) -> &str {
        match self {
            PlacingMode::Primary => "Primary",
            PlacingMode::Secondary => "Secondary",
            PlacingMode::CenterDot => "Center dot",
            _ => "Circle",
        }
    }

    fn default_stroke(&self) -> f32 {
        match self {
            PlacingMode::CenterDot => 2.0,
            _ => 3.0,
        }
    }

    fn make_circle(&self, center: egui::Pos2) -> Circle {
        Circle {
            center,
            radius: self.default_radius(),
            color: self.default_color(),
            label: self.default_label().to_string(),
            stroke_width: self.default_stroke(),
            visible: true,
        }
    }
}

// ── Drag state ────────────────────────────────────────────────────────────────

enum DragState {
    None,
    MovingCircle(usize),
    ResizingCircle(usize),
    MovingWindow,
}

impl Default for DragState {
    fn default() -> Self {
        DragState::None
    }
}

// ── Hit testing ───────────────────────────────────────────────────────────────

enum CircleHit {
    Body(usize),
    Edge(usize),
}

// ── App ───────────────────────────────────────────────────────────────────────

struct RcollApp {
    // Phase 2
    opacity: f32,
    hud_visible: bool,
    click_through: bool,
    // Phase 3
    circles: Vec<Circle>,
    selected: Option<usize>,
    placing_mode: PlacingMode,
    drag_state: DragState,
    context_circle: Option<usize>,
    rename_state: Option<(usize, String)>,
}

impl Default for RcollApp {
    fn default() -> Self {
        Self {
            opacity: 0.25,
            hud_visible: true,
            click_through: false,
            circles: Vec::new(),
            selected: None,
            placing_mode: PlacingMode::None,
            drag_state: DragState::None,
            context_circle: None,
            rename_state: None,
        }
    }
}

impl RcollApp {
    /// Hit-test circles from topmost (last) to bottommost (first).
    /// Edge = within ±8 px of circumference; Body = inside.
    fn hit_test(&self, pos: egui::Pos2) -> Option<CircleHit> {
        for (i, c) in self.circles.iter().enumerate().rev() {
            if !c.visible {
                continue;
            }
            let dist = (pos - c.center).length();
            if (dist - c.radius).abs() <= 8.0 {
                return Some(CircleHit::Edge(i));
            }
            if dist < c.radius - 8.0 {
                return Some(CircleHit::Body(i));
            }
        }
        None
    }

    /// Remove a circle and keep `selected` / `context_circle` consistent.
    fn remove_circle(&mut self, i: usize) {
        self.circles.remove(i);
        for slot in [&mut self.selected, &mut self.context_circle] {
            match *slot {
                Some(j) if j == i => *slot = None,
                Some(j) if j > i => *slot = Some(j - 1),
                _ => {}
            }
        }
        // Also fix drag state
        match self.drag_state {
            DragState::MovingCircle(j) | DragState::ResizingCircle(j) if j == i => {
                self.drag_state = DragState::None;
            }
            DragState::MovingCircle(ref mut j) | DragState::ResizingCircle(ref mut j)
                if *j > i =>
            {
                *j -= 1;
            }
            _ => {}
        }
    }
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

/// Draw a dashed circle with evenly spaced arcs.
fn draw_dashed_circle(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    stroke: egui::Stroke,
) {
    use std::f32::consts::TAU;
    const NUM_DASHES: usize = 16;
    const DASH_FRACTION: f32 = 0.45; // proportion of each slot that is a dash
    const STEPS: usize = 6; // line segments per dash arc

    for i in 0..NUM_DASHES {
        let t0 = (i as f32 / NUM_DASHES as f32) * TAU;
        let t1 = t0 + DASH_FRACTION / NUM_DASHES as f32 * TAU;
        let pts: Vec<egui::Pos2> = (0..=STEPS)
            .map(|j| {
                let t = t0 + (t1 - t0) * j as f32 / STEPS as f32;
                center + egui::vec2(radius * t.cos(), radius * t.sin())
            })
            .collect();
        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], stroke);
        }
    }
}

/// Draw a crosshair centred on `center` with arms of length `arm`.
fn draw_crosshair(
    painter: &egui::Painter,
    center: egui::Pos2,
    arm: f32,
    stroke: egui::Stroke,
) {
    painter.line_segment(
        [
            center - egui::vec2(arm, 0.0),
            center + egui::vec2(arm, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center - egui::vec2(0.0, arm),
            center + egui::vec2(0.0, arm),
        ],
        stroke,
    );
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Oxidized Optics",
        native_options,
        Box::new(|_cc| Ok(Box::new(RcollApp::default()))),
    )
}

impl eframe::App for RcollApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, self.opacity]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Global keyboard shortcuts ─────────────────────────────────────────
        let (h_pressed, esc_pressed) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::H),
                i.key_pressed(egui::Key::Escape),
            )
        });

        if h_pressed && !ctx.wants_keyboard_input() {
            self.hud_visible = !self.hud_visible;
        }
        if esc_pressed && !ctx.wants_keyboard_input() {
            if self.placing_mode.is_active() {
                // Cancel placement without quitting
                self.placing_mode = PlacingMode::None;
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // ── Keyboard nudges for selected circle ───────────────────────────────
        if let Some(sel) = self.selected {
            if sel < self.circles.len() && !ctx.wants_keyboard_input() {
                let (left, right, up, down, open_br, close_br, shift, ctrl) = ctx.input(|i| {
                    (
                        i.key_pressed(egui::Key::ArrowLeft),
                        i.key_pressed(egui::Key::ArrowRight),
                        i.key_pressed(egui::Key::ArrowUp),
                        i.key_pressed(egui::Key::ArrowDown),
                        i.key_pressed(egui::Key::OpenBracket),
                        i.key_pressed(egui::Key::CloseBracket),
                        i.modifiers.shift,
                        i.modifiers.ctrl,
                    )
                });

                let step: f32 = if shift {
                    0.1
                } else if ctrl {
                    5.0
                } else {
                    1.0
                };

                let c = &mut self.circles[sel];
                if left  { c.center.x -= step; }
                if right { c.center.x += step; }
                if up    { c.center.y -= step; }
                if down  { c.center.y += step; }
                if open_br  { c.radius = (c.radius - step).max(5.0); }
                if close_br { c.radius += step; }
            }
        }

        // ── HUD toolbar (top panel) ───────────────────────────────────────────
        if self.hud_visible {
            egui::TopBottomPanel::top("hud")
                .frame(
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgba_premultiplied(20, 20, 20, 220))
                        .inner_margin(egui::Margin::same(4)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // + Circle: toggles generic placement mode
                        let adding = self.placing_mode == PlacingMode::Generic;
                        if ui.selectable_label(adding, "+ Circle").clicked() {
                            self.placing_mode = if adding {
                                PlacingMode::None
                            } else {
                                PlacingMode::Generic
                            };
                        }

                        // Click-through toggle
                        let ct_label = if self.click_through {
                            "Click-thru: ON"
                        } else {
                            "Click-thru: OFF"
                        };
                        if ui.button(ct_label).clicked() {
                            self.click_through = !self.click_through;
                            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(
                                self.click_through,
                            ));
                        }

                        // Reset all circles
                        if ui.button("Reset").clicked() {
                            self.circles.clear();
                            self.selected = None;
                            self.context_circle = None;
                            self.placing_mode = PlacingMode::None;
                            self.drag_state = DragState::None;
                        }

                        // Phase 4 placeholders
                        if ui.button("Save").clicked() {}
                        if ui.button("Load").clicked() {}

                        ui.separator();

                        // Opacity slider
                        ui.label("Opacity:");
                        ui.add(
                            egui::Slider::new(&mut self.opacity, 0.0..=0.9)
                                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                                .custom_parser(|s| {
                                    s.trim_end_matches('%')
                                        .parse::<f64>()
                                        .ok()
                                        .map(|v| v / 100.0)
                                }),
                        );

                        // Right-aligned: hint + close
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button("✕").clicked() {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                ui.weak("H: toggle HUD  |  Esc: quit");
                            },
                        );
                    });
                });
        }

        // ── Circle list side panel ────────────────────────────────────────────
        if self.hud_visible {
            egui::SidePanel::left("circles_panel")
                .default_width(190.0)
                .frame(
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgba_premultiplied(20, 20, 20, 210))
                        .inner_margin(egui::Margin::same(6)),
                )
                .show(ctx, |ui| {
                    ui.heading("Circles");

                    // ── Preset buttons ────────────────────────────────────────
                    ui.horizontal_wrapped(|ui| {
                        let p = &self.placing_mode;
                        let placing_primary = *p == PlacingMode::Primary;
                        let placing_secondary = *p == PlacingMode::Secondary;
                        let placing_center = *p == PlacingMode::CenterDot;

                        if ui.selectable_label(placing_primary, "Primary").clicked() {
                            self.placing_mode = if placing_primary {
                                PlacingMode::None
                            } else {
                                PlacingMode::Primary
                            };
                        }
                        if ui.selectable_label(placing_secondary, "Secondary").clicked() {
                            self.placing_mode = if placing_secondary {
                                PlacingMode::None
                            } else {
                                PlacingMode::Secondary
                            };
                        }
                        if ui.selectable_label(placing_center, "Center dot").clicked() {
                            self.placing_mode = if placing_center {
                                PlacingMode::None
                            } else {
                                PlacingMode::CenterDot
                            };
                        }
                    });

                    if self.placing_mode.is_active() {
                        ui.colored_label(egui::Color32::YELLOW, "↓ Click canvas to place");
                    }

                    ui.separator();

                    // ── Circle list ───────────────────────────────────────────
                    let mut to_delete: Option<usize> = None;
                    let mut new_selected = self.selected;
                    let num_circles = self.circles.len();

                    if num_circles == 0 {
                        ui.weak("No circles yet.");
                    }

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for i in 0..num_circles {
                                let is_sel = new_selected == Some(i);
                                ui.horizontal(|ui| {
                                    // Visibility eye toggle
                                    let vis_icon =
                                        if self.circles[i].visible { "👁" } else { "○" };
                                    if ui.small_button(vis_icon).clicked() {
                                        self.circles[i].visible = !self.circles[i].visible;
                                    }

                                    // Color swatch (non-interactive, just visual)
                                    let (swatch_rect, _) = ui.allocate_exact_size(
                                        egui::vec2(12.0, 12.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter()
                                        .rect_filled(swatch_rect, 2.0, self.circles[i].color);

                                    // Label — click to select/deselect
                                    let resp = ui.selectable_label(is_sel, &self.circles[i].label);
                                    if resp.clicked() {
                                        new_selected = if is_sel { None } else { Some(i) };
                                    }

                                    // Delete button
                                    if ui.small_button("✕").clicked() {
                                        to_delete = Some(i);
                                    }
                                });
                            }
                        });

                    self.selected = new_selected;

                    if let Some(i) = to_delete {
                        self.remove_circle(i);
                    }

                    // ── Inspector for selected circle ─────────────────────────
                    if let Some(sel) = self.selected {
                        if sel < self.circles.len() {
                            ui.separator();
                            ui.label(egui::RichText::new("Inspector").strong());

                            // Color picker
                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                egui::color_picker::color_edit_button_srgba(
                                    ui,
                                    &mut self.circles[sel].color,
                                    egui::color_picker::Alpha::Opaque,
                                );
                            });

                            // Stroke width
                            ui.horizontal(|ui| {
                                ui.label("Stroke:");
                                ui.add(
                                    egui::DragValue::new(&mut self.circles[sel].stroke_width)
                                        .speed(0.1)
                                        .range(0.5..=12.0)
                                        .suffix(" px"),
                                );
                            });

                            ui.separator();

                            // Position numeric inputs
                            ui.label("Position:");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(
                                    egui::DragValue::new(&mut self.circles[sel].center.x)
                                        .speed(0.5)
                                        .suffix(" px"),
                                );
                                ui.label("Y:");
                                ui.add(
                                    egui::DragValue::new(&mut self.circles[sel].center.y)
                                        .speed(0.5)
                                        .suffix(" px"),
                                );
                            });

                            // Position nudge buttons (0.5 px per click)
                            ui.horizontal(|ui| {
                                if ui.small_button("←").clicked() {
                                    self.circles[sel].center.x -= 0.5;
                                }
                                if ui.small_button("→").clicked() {
                                    self.circles[sel].center.x += 0.5;
                                }
                                if ui.small_button("↑").clicked() {
                                    self.circles[sel].center.y -= 0.5;
                                }
                                if ui.small_button("↓").clicked() {
                                    self.circles[sel].center.y += 0.5;
                                }
                                ui.weak("0.5 px");
                            });

                            ui.separator();

                            // Radius numeric input
                            ui.horizontal(|ui| {
                                ui.label("Radius:");
                                ui.add(
                                    egui::DragValue::new(&mut self.circles[sel].radius)
                                        .speed(0.5)
                                        .range(5.0..=2000.0)
                                        .suffix(" px"),
                                );
                            });

                            // Radius nudge buttons (0.5 px per click)
                            ui.horizontal(|ui| {
                                if ui.small_button("−").clicked() {
                                    self.circles[sel].radius =
                                        (self.circles[sel].radius - 0.5).max(5.0);
                                }
                                if ui.small_button("+").clicked() {
                                    self.circles[sel].radius += 0.5;
                                }
                                ui.weak("0.5 px");
                            });

                            ui.add_space(2.0);
                            ui.weak("Arrows: ±1 px  Shift: ±0.1  Ctrl: ±5");
                            ui.weak("[ ] : radius ±1 px");
                        }
                    }
                });
        }

        // ── Central canvas ────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let canvas_rect = ui.available_rect_before_wrap();

                // Pointer position (hover, not necessarily pressed)
                let pointer_pos = ctx.input(|i| i.pointer.hover_pos());

                // Pre-compute hit under pointer for this frame
                let hit = pointer_pos.and_then(|pos| self.hit_test(pos));
                let hit_idx = match &hit {
                    Some(CircleHit::Body(i)) | Some(CircleHit::Edge(i)) => Some(*i),
                    None => None,
                };

                // Allocate the full canvas with click + drag sense
                let canvas_response =
                    ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

                // Record which circle was under the cursor on secondary click
                if canvas_response.secondary_clicked() {
                    self.context_circle = hit_idx;
                }

                // ── Placement mode ────────────────────────────────────────────
                if self.placing_mode.is_active() {
                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

                    if canvas_response.clicked() {
                        if let Some(pos) = canvas_response.interact_pointer_pos() {
                            let mode = self.placing_mode.clone();
                            let circle = mode.make_circle(pos);
                            self.circles.push(circle);
                            self.selected = Some(self.circles.len() - 1);
                            self.placing_mode = PlacingMode::None;
                        }
                    }
                } else {
                    // ── Normal canvas interaction ─────────────────────────────

                    // Precompute the grip rect so we can exclude it from window drag
                    let grip_size = 18.0;
                    let grip_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            canvas_rect.max.x - grip_size,
                            canvas_rect.max.y - grip_size,
                        ),
                        egui::Vec2::splat(grip_size),
                    );
                    let drag_start_on_grip = canvas_response
                        .interact_pointer_pos()
                        .map(|p| grip_rect.contains(p))
                        .unwrap_or(false);

                    if canvas_response.drag_started() && !drag_start_on_grip {
                        match &hit {
                            Some(CircleHit::Edge(i)) => {
                                self.selected = Some(*i);
                                self.drag_state = DragState::ResizingCircle(*i);
                            }
                            Some(CircleHit::Body(i)) => {
                                self.selected = Some(*i);
                                self.drag_state = DragState::MovingCircle(*i);
                            }
                            None => {
                                self.drag_state = DragState::MovingWindow;
                                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                            }
                        }
                    }

                    // Apply ongoing drag
                    let delta = canvas_response.drag_delta();
                    match &mut self.drag_state {
                        DragState::MovingCircle(idx) => {
                            let idx = *idx;
                            if idx < self.circles.len() {
                                self.circles[idx].center += delta;
                            }
                        }
                        DragState::ResizingCircle(idx) => {
                            let idx = *idx;
                            if idx < self.circles.len() {
                                if let Some(pos) = pointer_pos {
                                    let new_r =
                                        (pos - self.circles[idx].center).length();
                                    self.circles[idx].radius = new_r.max(5.0);
                                }
                            }
                        }
                        _ => {}
                    }

                    if canvas_response.drag_stopped() {
                        self.drag_state = DragState::None;
                    }

                    // Click (no drag) → select or deselect
                    if canvas_response.clicked() {
                        self.selected = hit_idx;
                    }

                    // Cursor hints based on what's under the pointer
                    match &hit {
                        Some(CircleHit::Edge(_)) => {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                        }
                        Some(CircleHit::Body(_)) => {
                            ctx.set_cursor_icon(egui::CursorIcon::Grab);
                        }
                        None => {}
                    }
                }

                // ── Context menu ──────────────────────────────────────────────
                let ctx_idx = self.context_circle;
                let ctx_label = ctx_idx
                    .and_then(|i| self.circles.get(i))
                    .map(|c| c.label.clone())
                    .unwrap_or_default();

                let mut rename_req = false;
                let mut duplicate_req = false;
                let mut delete_req = false;

                canvas_response.context_menu(|ui| {
                    if ctx_idx.is_some() {
                        ui.label(egui::RichText::new(&ctx_label).strong());
                        ui.separator();
                        if ui.button("Rename…").clicked() {
                            rename_req = true;
                            ui.close_menu();
                        }
                        if ui.button("Duplicate").clicked() {
                            duplicate_req = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Delete").clicked() {
                            delete_req = true;
                            ui.close_menu();
                        }
                    } else {
                        ui.weak("(no circle here)");
                    }
                });

                if let Some(i) = ctx_idx {
                    if rename_req && i < self.circles.len() {
                        self.rename_state = Some((i, self.circles[i].label.clone()));
                        self.selected = Some(i);
                    }
                    if duplicate_req && i < self.circles.len() {
                        let mut c = self.circles[i].clone();
                        c.center += egui::vec2(15.0, 15.0);
                        c.label.push_str(" (copy)");
                        self.circles.push(c);
                    }
                    if delete_req {
                        self.remove_circle(i);
                    }
                }

                // ── Draw circles ──────────────────────────────────────────────
                // Get painter after all ui.XXX() calls to satisfy the borrow checker
                let painter = ui.painter();
                for (i, c) in self.circles.iter().enumerate() {
                    if !c.visible {
                        continue;
                    }

                    let stroke = egui::Stroke::new(c.stroke_width, c.color);

                    if c.radius < 10.0 {
                        // Tiny circle: filled dot + crosshair arms
                        painter.circle_filled(c.center, c.radius.max(3.0), c.color);
                        draw_crosshair(painter, c.center, c.radius + 10.0, stroke);
                    } else {
                        painter.circle_stroke(c.center, c.radius, stroke);
                    }

                    // Selection highlight: dashed ring offset by 6 px
                    if self.selected == Some(i) {
                        let sel_stroke = egui::Stroke::new(
                            1.5,
                            egui::Color32::from_rgba_premultiplied(80, 180, 255, 210),
                        );
                        draw_dashed_circle(painter, c.center, c.radius + 6.0, sel_stroke);
                    }
                }

                // Ghost circle preview during placement
                if self.placing_mode.is_active() {
                    if let Some(pos) = pointer_pos {
                        let r = self.placing_mode.default_radius();
                        let [rv, gv, bv, _] = self.placing_mode.default_color().to_array();
                        let ghost =
                            egui::Color32::from_rgba_premultiplied(rv, gv, bv, 70);
                        painter.circle_stroke(
                            pos,
                            r,
                            egui::Stroke::new(1.5, ghost),
                        );
                        painter.circle_filled(pos, 3.5, ghost);
                    }
                }

                // ── Resize grip ───────────────────────────────────────────────
                let grip_size = 18.0;
                let grip_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        canvas_rect.max.x - grip_size,
                        canvas_rect.max.y - grip_size,
                    ),
                    egui::Vec2::splat(grip_size),
                );
                let grip_id = ui.id().with("resize_grip");
                let grip_response =
                    ui.interact(grip_rect, grip_id, egui::Sense::drag());
                if grip_response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(
                        egui::ResizeDirection::SouthEast,
                    ));
                }

                let grip_color = if grip_response.hovered() {
                    egui::Color32::from_rgba_premultiplied(220, 220, 220, 200)
                } else {
                    egui::Color32::from_rgba_premultiplied(160, 160, 160, 130)
                };
                for k in 1..=3usize {
                    let offset = k as f32 * 5.0;
                    painter.line_segment(
                        [
                            egui::pos2(canvas_rect.max.x - offset, canvas_rect.max.y - 1.0),
                            egui::pos2(canvas_rect.max.x - 1.0, canvas_rect.max.y - offset),
                        ],
                        egui::Stroke::new(1.5, grip_color),
                    );
                }

                // HUD hidden hint
                if !self.hud_visible {
                    painter.text(
                        canvas_rect.center_top() + egui::vec2(0.0, 14.0),
                        egui::Align2::CENTER_TOP,
                        "Press H to show toolbar",
                        egui::FontId::proportional(13.0),
                        egui::Color32::from_rgba_premultiplied(200, 200, 200, 120),
                    );
                }
            });

        // ── Rename dialog ─────────────────────────────────────────────────────
        let mut rename_confirmed = false;
        let mut rename_cancelled = false;

        if let Some((_idx, buf)) = &mut self.rename_state {
            egui::Window::new("Rename circle")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    let resp = ui.text_edit_singleline(buf);
                    // Auto-focus the text field when the dialog opens
                    resp.request_focus();

                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            rename_confirmed = true;
                        }
                        if ui.button("Cancel").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            rename_cancelled = true;
                        }
                    });
                });
        }

        if rename_confirmed {
            if let Some((idx, buf)) = self.rename_state.take() {
                if idx < self.circles.len() {
                    self.circles[idx].label = buf;
                }
            }
        } else if rename_cancelled {
            self.rename_state = None;
        }
    }
}
