use crate::windows::main_app::add_param;
use crate::wrap_app::alloc_ui_block;
use eframe::emath::Align;
use egui::text::LayoutJob;
use egui::{Color32, DragValue, FontId, Hyperlink, RichText, Stroke, TextFormat, Ui, Vec2};
use egui_plotter::EguiBackend;
use fresnel::fresnl;
use plotters::prelude::*;
use plotters::prelude::full_palette::GREEN_900;
use plotters::style::full_palette::ORANGE;

const US: i32 = 40;

pub struct DocApp {
    x1: f64,
    x2: f64,

    u1: f64,
    u2: f64,

    l1: f64,
    l2: f64,
    lambda: f64,

    #[cfg(debug_assertions)]
    p: i32,
}

impl DocApp {
    #[inline]
    fn draw_karnu(&mut self, ui: &Ui, u1: f64, u2: f64) -> [(f64, f64); 2] {
        let size = ui.available_height();
        let center = (size / 2.) as i32;
        let k_axis = center as f32 / 8.;

        let root = EguiBackend::new(ui).into_drawing_area();

        if ui.visuals().dark_mode {
            root.fill(&crate::windows::main_app::BG_PLOT_COLOR_DARK)
                .unwrap();
        } else {
            root.fill(&crate::windows::main_app::BG_PLOT_COLOR_LIGHT)
                .unwrap();
        }

        const AXIS: f64 = 0.8;
        let axis = (-AXIS..AXIS).step(0.1);
        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(axis.clone(), axis)
            .unwrap();

        chart
            .configure_mesh()
            .light_line_style(BLACK.mix(0.15))
            .max_light_lines(3)
            .draw()
            .unwrap();

        // draw axis
        {
            const LEN_SHTR: i32 = 4;
            let text_font = TextStyle::from(("sans-serif", 16).into_font());
            for i in (-7..=7).map(|x| x as f32) {
                if i != 0. {
                    let p = center + (k_axis * i).round() as i32;
                    root.draw(&PathElement::new(
                        [(center - LEN_SHTR, p), (center + LEN_SHTR, p)],
                        &BLACK,
                    ))
                    .unwrap();

                    root.draw_text(
                        format!("{}", i / 10.).as_str(),
                        &text_font,
                        (center + 7, p - 10),
                    )
                    .unwrap();

                    root.draw(&PathElement::new(
                        [(p, center - LEN_SHTR), (p, center + LEN_SHTR)],
                        &BLACK,
                    ))
                    .unwrap();

                    root.draw_text(
                        format!("{}", i / 10.).as_str(),
                        &text_font,
                        (p - 11, center + 4),
                    )
                    .unwrap();
                }
            }
            root.draw_text("0", &text_font, (center, center + 4))
                .unwrap();

            root.draw(&PathElement::new(
                [(center, 0), (center, 2 * center)],
                &BLACK,
            ))
            .unwrap();
            root.draw(&PathElement::new(
                [(0, center), (2 * center, center)],
                &BLACK,
            ))
            .unwrap();

            root.draw_text("C", &text_font, (2 * center - 12, center - 22))
                .unwrap();
            root.draw_text("S", &text_font, (center - 16, 0)).unwrap();

            let cnt = center as f64;
            let k = cnt / 0.8;
            for i in -4..=4 {
                if i == 0 {
                    continue;
                }
                let u = i as f64 / 2.;
                let (c, s) = fresnl(u);

                let txt = if u % 1. == 0. {
                    format!("{u:.0}")
                } else {
                    format!("{u:.2}")
                };

                chart
                    .draw_series(std::iter::once(Circle::new((s, c), 3, BLACK.filled())))
                    .unwrap();

                root.draw_text(
                    &txt,
                    &text_font,
                    ((cnt + s * k) as i32, (cnt - c * k) as i32),
                )
                .unwrap();
            }
        }

        chart
            .draw_series(LineSeries::new(
                ((-US * 10)..=(US * 10)).map(|u| {
                    let u = u as f64 / 20.;
                    let (s, c) = fresnl(u);
                    (c, s)
                }),
                &BLUE,
            ))
            .unwrap();

        let (s, c) = fresnl(u1);
        chart
            .draw_series(LineSeries::new([(c, 0.0), (c, s)], &BLACK))
            .unwrap();

        chart
            .draw_series(LineSeries::new([(0.0, s), (c, s)], &BLACK))
            .unwrap();
        chart
            .draw_series(LineSeries::new([(0.0, 0.0), (c, s)], &BLACK))
            .unwrap();


        let p1 = (c, s);

        let (s, c) = fresnl(u2);

        chart
            .draw_series(LineSeries::new([(c, 0.0), (c, s)], &BLACK))
            .unwrap();

        chart
            .draw_series(LineSeries::new([(0.0, s), (c, s)], &BLACK))
            .unwrap();
        chart
            .draw_series(LineSeries::new([(0.0, 0.0), (c, s)], &BLACK))
            .unwrap();

        let p2 = (c, s);

        chart
            .draw_series(LineSeries::new([p1, p2], &GREEN_900))
            .unwrap();


        chart
            .draw_series(std::iter::once(Circle::new(p1, 3, RED.filled())))
            .unwrap();

        chart
            .draw_series(std::iter::once(Circle::new(p2, 3, ORANGE.filled())))
            .unwrap();
        root.present().unwrap();
        [p1, p2]
    }

    #[inline]
    fn k(&self) -> f64 {
        (2. * (self.l1 + self.l2) / (self.lambda * self.l1 * self.l2)).sqrt()
    }

    #[inline]
    fn update_u(&mut self) {
        let k = self.k();
        self.u1 = self.x1 * k;
        self.u2 = self.x2 * k;
    }
}

impl eframe::App for DocApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(Align::LEFT), |ui| {
                let points =
                    ui.with_layout(egui::Layout::bottom_up(Align::LEFT), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("l1");
                            let r = ui.add(DragValue::new(&mut self.l1).suffix("см").speed(0.05));
                            if r.changed() {
                                self.update_u();
                            }
                            ui.label("l2:");
                            let r = ui.add(DragValue::new(&mut self.l2).suffix("см").speed(0.05));
                            if r.changed() {
                                self.update_u();
                            }

                            ui.label("λ:");
                            let r = ui.add(DragValue::new(&mut self.lambda).suffix("см").speed(0.05));
                            if r.changed() {
                                self.update_u();
                            }
                        });

                        let h = ui.available_height();
                        let size = Vec2::splat(h);
                        let inner_ui = &mut alloc_ui_block(ui, size);
                        self.draw_karnu(inner_ui, self.u1, self.u2)
                    }).inner;

                ui.vertical(|ui| {
                    {
                        ui.heading("В качестве аргумента, для U и X, так же можно вписать \"inf\" и \"-inf\"");
                        let drg = DragValue::new(&mut self.x1).suffix("см").speed(0.05);
                        let ch = add_param(ui, "x1: ", drg);
                        if ch {
                            self.update_u();
                        }

                        let drg = DragValue::new(&mut self.x2).suffix("см").speed(0.05);
                        let ch = add_param(ui, "x2: ", drg);
                        if ch {
                            self.update_u();
                        }

                        // let drg = DragValue::new(&mut self.l1).suffix("см").speed(0.05);
                        // let ch = add_param(ui, "l1:", drg);
                        // if ch {
                        //     self.update_u();
                        // }
                        //
                        // let drg = DragValue::new(&mut self.l2).suffix("см").speed(0.05);
                        // let ch = add_param(ui, "l2:", drg);
                        // if ch {
                        //     self.update_u();
                        // }
                        //
                        // let drg = DragValue::new(&mut self.lambda).suffix("см").speed(0.05);
                        // let ch = add_param(ui, "λ: ", drg);
                        // if ch {
                        //     self.update_u();
                        // }

                        let drg = DragValue::new(&mut self.u1).speed(0.05);
                        let ch = add_param(ui, "u1:", drg);
                        if ch {
                            self.x1 = self.u1 / self.k();
                        }
                        let drg = DragValue::new(&mut self.u2).speed(0.05);
                        let ch = add_param(ui, "u2:", drg);
                        if ch {
                            self.x2 = self.u2 / self.k();
                        }

                        ui.separator();

                        for (i, (c, s)) in points.iter().enumerate() {
                            let plus = if *s < 0. {
                                ""
                            } else {
                                "+"
                            };
                            ui.heading(
                                format!("C+Sj={c:.03}{plus}{s:.03}j\t|F{}|={:.3}", i, c.hypot(*s))
                            );
                        }
                        let c = (points[0].0 - points[1].0).abs();
                        let s = (points[0].1 - points[1].1).abs();
                        ui.heading(format!("|F|={:.3}", c.hypot(s)));

                        #[cfg(debug_assertions)]
                        {
                            let drag = DragValue::new(&mut self.p);
                            ui.add(drag);
                        }
                    }

                    ui.separator();

                    egui::ScrollArea::both().show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let mut job = LayoutJob::default();

                            let (default_color, strong_color) = if ui.visuals().dark_mode {
                                (Color32::DARK_GRAY, Color32::WHITE)
                            } else {
                                (Color32::DARK_GRAY, Color32::BLACK)
                            };

                            const TEXT_SIZE: f32 = 16.0;
                            // let TEXT_SIZE: f32 = self.p as f32;

                            job.append(
                                "\tДля построениия дифракционного множителя необходимо посчитать разность двух",
                                10.,
                                TextFormat {
                                    color: default_color,
                                    font_id: FontId::proportional(TEXT_SIZE),
                                    ..Default::default()
                                },
                            );
                            ui.label(job);

                            let h = Hyperlink::from_label_and_url(
                                RichText::new("интеграллов Френеля").size(TEXT_SIZE),
                                "https://ru.wikipedia.org/wiki/%D0%98%D0%BD%D1%82%D0%B5%D0%B3%D1%80%D0%B0%D0%BB%D1%8B_%D0%A4%D1%80%D0%B5%D0%BD%D0%B5%D0%BB%D1%8F",
                            );
                            ui.add(h).on_hover_text("Wikipedia");

                            let mut job = LayoutJob::default();
                            job.append(
                                "S(u) и C(u) (интеграллы по синусу и косинусу соответственно). Для их нахождения используется",
                                10.,
                                TextFormat {
                                    color: default_color,
                                    font_id: FontId::proportional(TEXT_SIZE),
                                    ..Default::default()
                                },
                            );
                            job.append(
                                "спираль Корню.",
                                10.,
                                TextFormat {
                                    color: default_color,
                                    font_id: FontId::proportional(TEXT_SIZE),
                                    underline: Stroke::new(1.0, strong_color),
                                    ..Default::default()
                                },
                            );
                            job.append(
                                "\n\tСпираль имеет свои координаты u, от 0 (где и начало всех координат) до ±∞ (где стираль закручивается). Подставляя свою u на спираль мы ставим точку на этой спирали и получаем точку, расположенную в плоскости с осями S и C. После чего визуально (линейкой) ищем какому S и какому C соответствует наша точка u. Её координаты есть S и C.",
                                10.,
                                TextFormat {
                                    color: default_color,
                                    font_id: FontId::proportional(TEXT_SIZE),
                                    ..Default::default()
                                },
                            );
                            ui.label(job);
                        });
                    });
                });
            })
        });
    }
}

impl Default for DocApp {
    fn default() -> Self {
        Self {
            x1: 0.,
            x2: 0.,
            u1: 0.,
            u2: 0.,
            l1: 30.,
            l2: 30.,
            lambda: 3.,
            #[cfg(debug_assertions)]
            p: 10,
        }
    }
}
