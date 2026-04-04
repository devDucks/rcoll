#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

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
        "rcoll",
        native_options,
        Box::new(|_cc| Ok(Box::new(RcollApp::default()))),
    )
}

struct RcollApp {
    /// Background fill alpha, 0.0 (transparent) to 0.9 (nearly opaque).
    opacity: f32,
    /// Whether the HUD toolbar is visible.
    hud_visible: bool,
    /// Whether mouse events pass through the window to whatever is behind it.
    click_through: bool,
}

impl Default for RcollApp {
    fn default() -> Self {
        Self {
            opacity: 0.25,
            hud_visible: true,
            click_through: false,
        }
    }
}

impl eframe::App for RcollApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Background fill alpha is driven by the opacity slider.
        // Circles (Phase 3) will be drawn at full alpha on top of this.
        [0.0, 0.0, 0.0, self.opacity]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Keyboard shortcuts ---
        let (h_pressed, esc_pressed) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::H),
                i.key_pressed(egui::Key::Escape),
            )
        });

        if h_pressed {
            self.hud_visible = !self.hud_visible;
        }
        if esc_pressed {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // --- HUD toolbar ---
        if self.hud_visible {
            egui::TopBottomPanel::top("hud")
                .frame(
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgba_premultiplied(20, 20, 20, 220))
                        .inner_margin(egui::Margin::same(4)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // Add circle — placeholder for Phase 3
                        if ui.button("+ Circle").clicked() {
                            // Phase 3: place a new collimation circle
                        }

                        // Click-through toggle
                        let ct_label = if self.click_through {
                            "Click-thru: ON"
                        } else {
                            "Click-thru: OFF"
                        };
                        if ui.button(ct_label).clicked() {
                            self.click_through = !self.click_through;
                            ctx.send_viewport_cmd(
                                egui::ViewportCommand::MousePassthrough(self.click_through),
                            );
                        }

                        // Reset — placeholder for Phase 3
                        if ui.button("Reset").clicked() {
                            // Phase 3: clear all circles
                        }

                        // Save / Load — placeholders for Phase 4
                        if ui.button("Save").clicked() {
                            // Phase 4: save layout to JSON
                        }
                        if ui.button("Load").clicked() {
                            // Phase 4: load layout from JSON
                        }

                        ui.separator();

                        // Opacity slider
                        ui.label("Opacity:");
                        ui.add(
                            egui::Slider::new(&mut self.opacity, 0.0..=0.9)
                                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                                .custom_parser(|s| s.trim_end_matches('%').parse::<f64>().ok().map(|v| v / 100.0)),
                        );

                        // Right-aligned: hint and close button
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

        // --- Main canvas ---
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let canvas_rect = ui.available_rect_before_wrap();

                // Drag the whole window by clicking/dragging anywhere on the canvas.
                let drag_response =
                    ui.allocate_rect(canvas_rect, egui::Sense::drag());
                if drag_response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                // Resize grip in the bottom-right corner.
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

                // Draw the resize grip as diagonal tick marks.
                let painter = ui.painter();
                let grip_color = if grip_response.hovered() {
                    egui::Color32::from_rgba_premultiplied(220, 220, 220, 200)
                } else {
                    egui::Color32::from_rgba_premultiplied(160, 160, 160, 130)
                };
                for i in 1..=3usize {
                    let offset = i as f32 * 5.0;
                    painter.line_segment(
                        [
                            egui::pos2(canvas_rect.max.x - offset, canvas_rect.max.y - 1.0),
                            egui::pos2(canvas_rect.max.x - 1.0, canvas_rect.max.y - offset),
                        ],
                        egui::Stroke::new(1.5, grip_color),
                    );
                }

                // When the HUD is hidden, show a small hint so the user can
                // recover it without knowing the keyboard shortcut.
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
    }
}
