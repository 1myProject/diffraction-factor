use egui::special_emojis::GITHUB;
use egui::Vec2;

#[derive(Default, PartialEq)]
enum SizeType {
    #[default]
    H800,
    H1080,
    H1200,
    H1440,
    H1600,
}

impl SizeType {
    pub fn get_hight(&self) -> f32 {
        match self {
            SizeType::H800 => MAIN_HEIGHT,
            SizeType::H1080 => 700.,
            SizeType::H1200 => 900.,
            SizeType::H1440 => 1100.,
            SizeType::H1600 => 1300.,
        }
    }
}

#[derive(Default)]
pub struct SettingsApp {
    size_type: SizeType,
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("настройка размера окна для экрана:");
            ui.selectable_value(&mut self.size_type, SizeType::H800, "800px");
            ui.selectable_value(&mut self.size_type, SizeType::H1080, "1080px");
            ui.selectable_value(&mut self.size_type, SizeType::H1200, "1200px");
            ui.selectable_value(&mut self.size_type, SizeType::H1440, "1440px");
            ui.selectable_value(&mut self.size_type, SizeType::H1600, "1600px");

            let y = self.size_type.get_hight();

            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(compute_window_size(y)));

            // info block
            ui.separator();
            ui.heading("Информация");

            ui.horizontal(|ui| {
                ui.label("Счетовод:");
                ui.hyperlink_to(
                    format!("{GITHUB}MorinoSenshi"),
                    "https://github.com/1myProject",
                );
            });
            ui.horizontal(|ui| {
                ui.label("Продакшн: ");
                ui.hyperlink_to(
                    format!("{GITHUB}Prokoptonator"),
                    "https://github.com/Prokoptonator",
                );
            });

            ui.add_space(12.0);

            ui.label("программа написана на Rust в 2025г");
            ui.hyperlink_to(
                format!("{GITHUB}Исходный код"),
                "https://github.com/1myProject/Huygens-Fresnel-egui-plot",
            );

        });
    }
}


pub const COLS: f32 = 10.;
pub const ROWS: f32 = 6.;

pub const OFFEST_Y: f32 = 77.;
pub const OFFEST_X: f32 = 16.;

pub const MAIN_HEIGHT: f32 = 600.;
pub fn compute_window_size(hight: f32) -> Vec2 {
    Vec2::new(hight / ROWS * COLS + OFFEST_X, hight + OFFEST_Y)
}
