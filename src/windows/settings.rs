use egui::special_emojis::GITHUB;
use egui::Vec2;

///settings window (size of window & scale of elements)


/// hight of window
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

/// scales of pixels 
#[derive(Default, PartialEq)]
enum ScaleType{
    #[default]
    S1,
    S2,
    S3,
    S4
}

impl ScaleType {
    pub fn get_dpi(&self) -> f32 {
        match self {
            ScaleType::S1 => 1.,
            ScaleType::S2 => 1.2,
            ScaleType::S3 => 1.4,
            ScaleType::S4 => 1.6,
        }
    }
}
#[derive(Default)]
pub struct SettingsApp {
    size_type: SizeType,
    scale_type: ScaleType,
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("настройка размера окна для экрана:");
                    ui.selectable_value(&mut self.size_type, SizeType::H800, "800px");
                    ui.selectable_value(&mut self.size_type, SizeType::H1080, "1080px");
                    ui.selectable_value(&mut self.size_type, SizeType::H1200, "1200px");
                    ui.selectable_value(&mut self.size_type, SizeType::H1440, "1440px");
                    ui.selectable_value(&mut self.size_type, SizeType::H1600, "1600px");
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.heading("настройка маштаба:");
                    ui.selectable_value(&mut self.scale_type, ScaleType::S1, "100%");
                    ui.selectable_value(&mut self.scale_type, ScaleType::S2, "120%");
                    ui.selectable_value(&mut self.scale_type, ScaleType::S3, "140%");
                    ui.selectable_value(&mut self.scale_type, ScaleType::S4, "160%");
                })
            });

            //updater os settings
            
            let sc = self.scale_type.get_dpi();

            // print!("\r{}", ctx.pixels_per_point());
            ctx.set_pixels_per_point(sc); // set scale of pixels

            let y = self.size_type.get_hight()/sc; // set hight for new scale

            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(compute_window_size(y)));
            
            // info block
            
            ui.separator();
            ui.heading("Информация");

            ui.label("программа написана на Rust в 2025г, под руководством доцента кафедры ИРТ Кирильчука В.Б.");

            ui.horizontal(|ui| {
                ui.label("Счетовод: Мурин В.Д. ");
                ui.hyperlink_to(
                    format!("{GITHUB}MorinoSenshi"),
                    "https://github.com/1myProject",
                );
            });
            ui.horizontal(|ui| {
                ui.label("Продакшн: Прокопчик Д.В. ");
                ui.hyperlink_to(
                    format!("{GITHUB}Prokoptonator"),
                    "https://github.com/Prokoptonator",
                );
            });
            ui.horizontal(|ui| {
                ui.label("Ученый: Жуковский П.Н.");
            });

            ui.add_space(12.0);
            ui.hyperlink_to(
                format!("{GITHUB}Исходный код"),
                "https://github.com/1myProject/diffraction-factor",
            );

        });
    }
}

// count of blocks for height (rows) and width (cols)
pub const COLS: f32 = 10.;
pub const ROWS: f32 = 6.;

// the top title bar is also part of the window
pub const OFFEST_Y: f32 = 77.;
pub const OFFEST_X: f32 = 16.; // I don't know why but it needed

pub const MAIN_HEIGHT: f32 = 600.;
pub fn compute_window_size(hight: f32) -> Vec2 {
    Vec2::new(hight / ROWS * COLS + OFFEST_X, hight + OFFEST_Y)
}
