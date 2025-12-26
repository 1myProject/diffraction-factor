use crate::windows::main_app::MainApp;
use crate::windows::doc_app::DocApp;
use eframe::emath::Vec2;
use egui::{Ui, UiBuilder, ViewportCommand, Visuals};
#[cfg(debug_assertions)]
use std::time::{Duration, Instant};
use crate::windows::settings::SettingsApp;

/// mark for current window
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Anchor {
    #[default]
    Main,
    Doc,
    Setting,
}

// ----------------------------------------------------------------------------
/// exemplars of inner windows
#[derive(Default)]
pub struct State {
    main: MainApp,
    doc: DocApp,
    settings: SettingsApp,

    selected_anchor: Anchor,
}

///main window
#[derive(Default)]
pub struct WrapApp {
    pub state: State,

    // for time of update frame
    #[cfg(debug_assertions)]
    last_time_run: Duration,

    #[cfg(debug_assertions)]
    cpu_usage: Option<f32>,
}

impl WrapApp {
    /// init function
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx); // for svg images
        Default::default()
    }

    /// function for list to top bar
    pub fn apps_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&'static str, Anchor, &mut dyn eframe::App)> {
        let vec = vec![
            (
                "Главная",
                Anchor::Main,
                &mut self.state.main as &mut dyn eframe::App,
            ),
            (
                "Корню",
                Anchor::Doc,
                &mut self.state.doc as &mut dyn eframe::App,
            ),
            (
                "Настройки",
                Anchor::Setting,
                &mut self.state.settings as &mut dyn eframe::App,
            ),
        ];

        vec.into_iter()
    }
}

impl eframe::App for WrapApp {
    /// drawing of window
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(debug_assertions)]
        let st = Instant::now(); // start drawing

        let panel_frame = egui::Frame::new()
            .fill(ctx.style().visuals.window_fill())
            .inner_margin(4);

        // for top window
        egui::TopBottomPanel::top("wrap_app_top_bar")
            .frame(panel_frame)
            .show(ctx, |ui| {
                let app_rect = ui.max_rect();

                // alloc a size of top window
                let title_bar_height = 32.0;
                let title_bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + title_bar_height;
                    rect
                };
                // draw a top window
                title_bar_ui(ui, title_bar_rect, "Дифракция");

                // draw a top list of windows buttons
                ui.horizontal_wrapped(|ui| {
                    ui.visuals_mut().button_frame = true;
                    self.bar_contents(ui);
                });
            });

        self.show_selected_app(ctx, frame);

        // end of drawing
        #[cfg(debug_assertions)]
        {
            self.last_time_run = st.elapsed();
            self.cpu_usage = frame.info().cpu_usage;
        }
    }

    //from egui demo. I don't know what it do
    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        // Give the area behind the floating windows a different color, because it looks better:
        let color = egui::lerp(
            egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
            0.5,
        );
        let color = egui::Color32::from(color);
        color.to_normalized_gamma_f32()
    }
}

impl WrapApp {
    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected_anchor = self.state.selected_anchor;
        for (_name, anchor, app) in self.apps_iter_mut() {
            if anchor == selected_anchor || ctx.memory(|mem| mem.everything_is_visible()) {
                app.update(ctx, frame);
            }
        }
    }

    fn bar_contents(&mut self, ui: &mut Ui) {
        // theme button
        egui::widgets::global_theme_preference_switch(ui);

        ui.separator();

        let mut selected_anchor = self.state.selected_anchor;
        for (name, anchor, _app) in self.apps_iter_mut() {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
            }
        }
        self.state.selected_anchor = selected_anchor;

        // view debag info
        #[cfg(debug_assertions)]
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::warn_if_debug_build(ui);
            ui.label(format!("{:?}", self.last_time_run));
            if let Some(cpu) = self.cpu_usage {
                ui.label(format!("{:.3}%", cpu * 100.));
            }
        });
    }
}

/// it for main_app. There are alloc rect blocks for plots
pub fn alloc_ui_block(ui: &mut Ui, size: Vec2) -> Ui {
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::default()),
    )
}

fn title_bar_ui(ui: &mut Ui, title_bar_rect: eframe::epaint::Rect, title: &str) {
    use egui::{vec2, Align2, FontId, Id, PointerButton, Sense, UiBuilder};

    let painter = ui.painter();

    // for can move a window
    let title_bar_response = ui.interact(
        title_bar_rect,
        Id::new("title_bar"),
        Sense::click_and_drag(),
    );

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    // Paint the line under the title:
    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    if title_bar_response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);

            use egui::{Button, RichText};

            let button_height = 20.0;

            // crutch. it from egui demo
            let close_response = ui.add(Button::new(RichText::new("❌").size(button_height)));
            if close_response.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }

            let minimized_response = ui.add(Button::new(RichText::new("🗕").size(button_height)));
            if minimized_response.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
            }
        },
    );
}
