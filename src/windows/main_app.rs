use crate::windows::math::calc::converter_freq;
use crate::windows::math::chart::ChartParams;
use crate::windows::math::difr::{Difr, Screens, MAX_X};
use crate::windows::settings::{COLS, OFFEST_X, OFFEST_Y, ROWS};
use crate::wrap_app::alloc_ui_block;
use egui::{Button, Color32, DragValue, Event, Image, Pos2, Rect, Ui, Vec2};
use egui_plot::{AxisHints, GridInput, GridMark, Line, MarkerShape, Plot, PlotPoint, Points};
use egui_plotter::EguiBackend;
use plotters::prelude::*;
use plotters::style::full_palette::GREY_500;
use std::default::Default;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, SQRT_2};
use std::ops::RangeInclusive;

const MOVE_SCALE: f32 = 0.01;
pub(crate) const BG_PLOT_COLOR_DARK: RGBColor = GREY_500;
pub(crate) const BG_PLOT_COLOR_LIGHT: RGBColor = WHITE;

const COLOR_LINE: RGBColor = BLUE;
const COLOR_GRID_LIGHT: RGBColor = RGBColor(204, 204, 204);
const COLOR_GRID_DARK: RGBColor = RGBColor(122, 122, 122);
const COLOR_PROEKCIA_LILGHT: RGBColor = RGBColor(0, 139, 0);
const COLOR_PROEKCIA_DARK: RGBColor = RGBColor(173, 255, 173);
const COLOR_RED_POINT: RGBColor = RGBColor(212, 0, 0);
const COLOR_RED_POINT_EGUI: Color32 = Color32::from_rgb(212, 0, 0);

#[derive(PartialEq, Default)]
enum ScreenMod {
    #[default]
    Rectangle,
    Circle,
}

#[derive(Default)]
pub struct MainApp {
    chart_params: ChartParams,

    fz: Difr,
    is_freq: bool,
    zoom: bool,
    screen_mod: ScreenMod,

    #[cfg(debug_assertions)]
    p: f64,
}

impl MainApp {
    fn x_grid(input: GridInput) -> Vec<GridMark> {
        // Note: this always fills all possible marks. For optimization, `input.bounds`
        // could be used to decide when the low-interval grids (minutes) should be added.

        let mut marks = vec![];

        let (min, max) = input.bounds;

        let preselect = if max - min < 2. { 10. } else { 1. };

        let step = ((max - min) / 7. * preselect).ceil();

        let min = (min * preselect).floor() as i32 + 1;
        let max = (max * preselect).ceil() as i32;
        for i in min..max {
            let step_size = if i % step as i32 == 0 {
                // 1 day
                step / preselect
            } else if i % (step / 2.) as i32 == 0 {
                // 1 hour
                step / 2. / preselect
            } else {
                continue;
            };

            let value = (i as f64) / preselect;

            marks.push(GridMark {
                value,
                step_size,
                // step_size: 1.0
            });
        }

        // let min = min.floor() as i32;
        // let max = max.ceil() as i32;
        // for i in min..=max {
        //     let step_size = if i % Self::MINS_PER_DAY as i32 == 0 {
        //         // 1 day
        //         Self::MINS_PER_DAY
        //     } else if i % Self::MINS_PER_H as i32 == 0 {
        //         // 1 hour
        //         Self::MINS_PER_H
        //     } else {
        //         1.0
        //         // skip grids below 5min
        //         // continue;
        //     };
        //
        //     print!("({:.2}, {:.2}), ", i, step_size);
        //     marks.push(GridMark {
        //         value: i as f64,
        //         step_size,
        //         // step_size: 1.0
        //     });
        // }

        marks
    }
}

impl MainApp {
    fn draw_3d(&mut self, ui: &mut Ui, line: Vec<(f64, f64, f64)>) {
        let available = ui.available_size();

        {
            let center = available.x / 2.;
            let rebro_05 = center * 0.7;
            let r = available.x * 0.7 / SQRT_2;

            let rect = Rect::from_min_size(Pos2::new(0., 0.), available);
            // Регистрируем область для взаимодействия, чувствительную к клику и перетаскиванию
            let response = ui.interact(
                rect,
                ui.id().with("some_unique_id"),
                egui::Sense::click_and_drag(),
            );

            let chpr = &mut self.chart_params;

            // get mouse data
            let (pitch_delta, yaw_delta) = ui.input(|input| {
                // Проверяем, находится ли курсор над областью
                if !response.hovered() {
                    return (0.0, 0.0);
                }
                let pointer = &input.pointer;
                let delta = pointer.delta();

                let (pitch_delta, yaw_delta) = match pointer.primary_down() {
                    true => (delta.y * MOVE_SCALE, -delta.x * MOVE_SCALE),
                    false => (0., 0.),
                };
                // println!("pitch: {:<5.2}|yaw {:<5.2}", chpr.pitch, chpr.yaw);
                (pitch_delta, yaw_delta)
            });

            chpr.pitch_vel = pitch_delta;
            chpr.yaw_vel = yaw_delta;

            chpr.pitch += chpr.pitch_vel;
            chpr.yaw += chpr.yaw_vel;

            const ZERS: f32 = 100.;
            chpr.yaw = (chpr.yaw * ZERS).round() / ZERS;
            chpr.pitch = (chpr.pitch * ZERS).round() / ZERS;

            if chpr.pitch >= FRAC_PI_2 {
                chpr.pitch = FRAC_PI_2;
            } else if chpr.pitch <= -FRAC_PI_2 {
                chpr.pitch = -FRAC_PI_2;
            }

            // Next plot everything
            let root = EguiBackend::new(ui).into_drawing_area();

            if ui.visuals().dark_mode {
                root.fill(&BG_PLOT_COLOR_DARK).unwrap();
            } else {
                root.fill(&BG_PLOT_COLOR_LIGHT).unwrap();
            }

            // crutch. egui-plotter does not support axis labels in 3d graphs
            {
                let text_font = TextStyle::from(("sans-serif", 20).into_font());

                // S axis
                let ang = (chpr.yaw % FRAC_PI_2).abs() - FRAC_PI_4;
                let xl = center - r * ang.abs().cos();
                let mut yl = r * ang.sin() * chpr.pitch.sin();
                if chpr.yaw < 0. {
                    yl = -yl;
                }
                root.draw_text(
                    "S",
                    &text_font,
                    (xl.round() as i32, (center - yl).round() as i32),
                )
                .unwrap();

                // u axis
                let phi = chpr.pitch % PI;

                let part1 = if chpr.pitch >= 0. {
                    rebro_05 * chpr.pitch.cos() * chpr.pitch.cos()
                } else {
                    -rebro_05 * chpr.pitch.cos() * chpr.pitch.cos() - available.x * 0.05
                };
                let part2 = rebro_05 * phi.sin();

                let mut theta = (chpr.yaw + FRAC_PI_2) % PI;
                if theta < 0. {
                    theta += PI;
                }
                let xl = rebro_05 * theta.cos();
                let yl = part1 + part2 * theta.abs().sin();

                root.draw_text(
                    "u",
                    &text_font,
                    ((center + xl).round() as i32, (center + yl).round() as i32),
                )
                .unwrap();

                // C axis
                let mut theta = chpr.yaw % PI;
                if theta < 0. {
                    theta += PI;
                }
                let xl = rebro_05 * theta.cos();
                let yl = part1 + part2 * theta.abs().sin();
                // println!(
                //     "pitch: {:<5.2}|yaw {:<5.2}|theta {:<5.2}|phi {:<5.2}|x {:<5.2}|y {:<5.2}",
                //     chpr.pitch, chpr.yaw, theta, phi, xl, -yl
                // );

                root.draw_text(
                    "C",
                    &text_font,
                    ((center + xl).round() as i32, (center + yl).round() as i32),
                )
                .unwrap();
            }

            //setting axis
            const ZR: f64 = 10.;
            let st = if let Some(r) = line.first() {
                (r.0 * ZR).floor() / ZR
            } else {
                return;
            };
            let end = (line.last().unwrap().0 * ZR).ceil() / ZR;

            let step = (end - st).round() / 6.;
            let step = (step * 100.).round() / 100.;
            let x_axis = (st..end).step(step);
            let y_axis = (-1.0..1.0).step(0.1);
            let z_axis = (1.0..-1.0).step(-0.1);

            let mut chart = ChartBuilder::on(&root)
                .build_cartesian_3d(x_axis, y_axis, z_axis)
                .unwrap();

            //setting angil of view
            chart.with_projection(|mut pb| {
                pb.yaw = chpr.yaw as f64;
                pb.scale = 0.75;
                pb.pitch = chpr.pitch as f64;
                pb.into_matrix()
            });

            // setting of mesh
            chart
                .configure_axes()
                .light_grid_style(if ui.visuals().dark_mode {
                    COLOR_GRID_DARK
                } else {
                    COLOR_GRID_LIGHT
                })
                .max_light_lines(3)
                .draw()
                .unwrap();

            let points = match self.fz.rezhim {
                Screens::One => {
                    let mut v = vec![(0.0, 0.0, 0.0), self.fz.get_current_point_3d()];
                    chart
                        .draw_series(LineSeries::new(v.clone(), &BLACK))
                        .unwrap();
                    v.remove(0);
                    v
                }
                Screens::Two => {
                    let (point1, point2) = self.fz.get_current_points_3d();

                    let v = vec![point1, point2];
                    chart
                        .draw_series(LineSeries::new(v.clone(), &BLACK))
                        .unwrap();
                    v
                }
            };

            // draw red point
            chart
                .draw_series(
                    points
                        .iter()
                        .map(|&(x, y, z)| Circle::new((x, y, z), 3, COLOR_RED_POINT)),
                )
                .unwrap();

            // draw projection on the plot
            // I don't know who of whose
            {
                const POINT_PROJECTION_SIZE: i32 = 3;

                let mut phi = (chpr.yaw) % (2. * PI);
                if phi < 0. {
                    phi += 2. * 3.14;
                }

                // println!("{:>5}|{phi:>5}", chpr.yaw);

                let color_proekt = if ui.visuals().dark_mode {
                    COLOR_PROEKCIA_DARK
                } else {
                    COLOR_PROEKCIA_LILGHT
                };
                if phi != 3.14 && phi != 0. {
                    let p = if phi < PI {
                        -MAX_X as f64 * self.fz.k()
                    } else {
                        MAX_X as f64 * self.fz.k()
                    };

                    let l = line.iter().map(|&(_, y, z)| (p, y, z));
                    chart
                        .draw_series(DashedLineSeries::new(l, 5, 5, color_proekt.into()))
                        .unwrap();

                    for &(_, y, z) in points.iter() {
                        chart
                            .draw_series(LineSeries::new([(p, -1., z), (p, 1., z)], &BLACK))
                            .unwrap();
                        chart
                            .draw_series(LineSeries::new([(p, y, -1.), (p, y, 1.)], &BLACK))
                            .unwrap();
                    }

                    chart
                        .draw_series(points.iter().map(|&(_, y, z)| {
                            Circle::new((p, y, z), POINT_PROJECTION_SIZE, COLOR_RED_POINT.filled())
                        }))
                        .unwrap();
                }

                if chpr.pitch != 0. {
                    let p = if chpr.pitch > 0. { -1. } else { 1. };

                    let l = line.iter().map(|&(x, _, z)| (x, p, z));
                    chart
                        .draw_series(DashedLineSeries::new(l, 5, 5, color_proekt.into()))
                        .unwrap();

                    for &(x, _, z) in points.iter() {
                        chart
                            .draw_series(LineSeries::new([(x, p, -1.), (x, p, 1.)], &BLACK))
                            .unwrap();
                        chart
                            .draw_series(LineSeries::new([(st, p, z), (end, p, z)], &BLACK))
                            .unwrap();
                    }

                    chart
                        .draw_series(points.iter().map(|&(x, _, z)| {
                            Circle::new((x, p, z), POINT_PROJECTION_SIZE, COLOR_RED_POINT.filled())
                        }))
                        .unwrap();
                }

                let phi = phi - FRAC_PI_2;
                if phi != 0. && phi != 3.14 {
                    let p = -if phi > 0. && phi < PI { 1. } else { -1. };

                    let l = line.iter().map(|&(x, y, _)| (x, y, p));
                    chart
                        .draw_series(DashedLineSeries::new(l, 5, 5, color_proekt.into()))
                        .unwrap();

                    for &(x, y, _) in points.iter() {
                        chart
                            .draw_series(LineSeries::new([(x, -1., p), (x, 1., p)], &BLACK))
                            .unwrap();
                        chart
                            .draw_series(LineSeries::new([(st, y, p), (end, y, p)], &BLACK))
                            .unwrap();
                    }

                    chart
                        .draw_series(points.iter().map(|&(x, y, _)| {
                            Circle::new((x, y, p), POINT_PROJECTION_SIZE, COLOR_RED_POINT.filled())
                        }))
                        .unwrap();
                }
            }

            // main line function
            chart
                .draw_series(LineSeries::new(line.clone(), COLOR_LINE))
                .unwrap();

            root.present().unwrap();
        }

        // experimental
        #[cfg(debug_assertions)]
        {
            // if false then background is invisible
            let resp = if self.zoom {
                ui.selectable_label(true, "zoom")
            } else {
                ui.button("zoom")
            };

            if resp.clicked() {
                self.zoom = !self.zoom;
            };
        }
    }

    fn parameters(&mut self, ui: &mut Ui) {
        let fz = &mut self.fz;
        ui.vertical(|ui| {
            //Count of Screens
            ui.horizontal(|ui| {
                let (name, chang) = match fz.rezhim {
                    Screens::One => ("1", Screens::Two),
                    Screens::Two => ("2", Screens::One),
                };
                ui.label("количество экранов: ");
                if ui.button(name).clicked() {
                    fz.rezhim = chang;
                    fz.rebuild_integrals();
                    self.screen_mod = ScreenMod::Rectangle;
                }

                if Screens::One == fz.rezhim {
                    let (img, chang) = match self.screen_mod {
                        ScreenMod::Circle => (
                            egui::include_image!("../../imgs/rect.svg"),
                            ScreenMod::Rectangle,
                        ),
                        ScreenMod::Rectangle => (
                            egui::include_image!("../../imgs/circle.svg"),
                            ScreenMod::Circle,
                        ),
                    };

                    if ui.add(Button::image(Image::new(img))).clicked() {
                        self.screen_mod = chang
                    }
                }
            });

            //len from center to screen
            ui.horizontal(|ui| {
                let start = fz.get_start();

                ui.label("x:")
                    .on_hover_text("расстояние от центра окна до кромки экрана");
                ui.add(
                    DragValue::new(&mut fz.x_otv)
                        .range(start..=MAX_X)
                        .suffix("см")
                        .speed(0.1),
                )
                .on_hover_text("расстояние от центра окна до кромки экрана");
            });

            // frequency/len of vawe
            ui.horizontal(|ui| {
                let name = if self.is_freq { "f:" } else { "λ:" };
                ui.label(name);

                let drag = if self.is_freq {
                    DragValue::new(&mut fz.freq)
                        .range(6.0..=20.)
                        .suffix("ГГц")
                        .speed(0.1)
                } else {
                    DragValue::new(&mut fz.lambda)
                        .range(0.1..=5.)
                        .suffix("см")
                        .speed(0.1)
                };
                ui.add(drag);

                if self.is_freq {
                    fz.lambda = converter_freq(fz.freq);
                } else {
                    fz.freq = converter_freq(fz.lambda);
                }

                const MIN_LAMBDA: f32 = 0.1;
                if fz.lambda < MIN_LAMBDA {
                    fz.lambda = MIN_LAMBDA;
                }

                let name = if !self.is_freq {
                    "в ГГц"
                } else {
                    "в см"
                };
                if ui.button(name).clicked() {
                    self.is_freq = !self.is_freq;
                };
            });

            // len of rupr to screen
            let drag = DragValue::new(&mut fz.l1)
                .range(0.1..=50.)
                .suffix("см")
                .speed(0.1);
            add_param(ui, "📢 ~ ||     L1:", drag);

            //len of screen to rupr
            let drag = DragValue::new(&mut fz.l2)
                .range(0.1..=50.)
                .suffix("см")
                .speed(0.1);
            add_param(ui, "|| ~ 📢     L2:", drag);

            // helper
            #[cfg(debug_assertions)]
            {
                let drag = DragValue::new(&mut self.p).range(0.1..=50.).speed(0.1);
                add_param(ui, "p", drag);
            }
        });

        // detect of update of params
        if fz.is_cheng() {
            if fz.not_sale_cpu_usage() {
                fz.cheng_copes();
                fz.rebuild_integrals()
            } else {
                fz.backup_copes()
            }
        }
    }

    fn draw_wave(&mut self, ui: &mut Ui) {
        let root_size = ui.available_width();

        let root = EguiBackend::new(ui).into_drawing_area();

        let center = (root_size / 2.).round();
        let center_of_circle = (center as i32, center as i32);

        let root_k = center / MAX_X;

        // if screen is close
        if self.fz.get_start() == self.fz.x_otv {
            root.fill(&BLACK).unwrap();
            return;
        }

        if self.screen_mod == ScreenMod::Circle {
            root.fill(&BLACK).unwrap();
        }

        // radius of waves
        let waves = self
            .fz
            .get_fresnel_zones(self.screen_mod == ScreenMod::Circle)
            .into_iter();
        let max_n = waves.len() - 1;
        let mut last_r = 0.0;
        for (n, r) in waves.into_iter().enumerate() {
            if self.screen_mod == ScreenMod::Circle && r >= self.fz.x_otv {
                last_r = r;
                continue;
            }
            let color = if (n + max_n) % 2 == 0 { &BLUE } else { &RED };
            root.draw(&Circle::new(
                center_of_circle,
                last_r * root_k,
                ShapeStyle::from(color).filled(),
            ))
            .unwrap();
            last_r = r;
        }
        // its wave too
        root.draw(&Circle::new(
            center_of_circle,
            last_r * root_k,
            ShapeStyle::from(&RED).filled(),
        ))
        .unwrap();

        // draw screen
        match self.fz.rezhim {
            Screens::One => {
                if self.screen_mod == ScreenMod::Rectangle {
                    let x1 = center + self.fz.x_otv * root_k;
                    let x2 = (center + root_k * (MAX_X + 1.)) as i32;
                    root.draw(&Rectangle::new(
                        [(x1 as i32, 0), (x2, x2)],
                        ShapeStyle::from(BLACK.filled()),
                    ))
                    .unwrap();
                }
            }
            Screens::Two => {
                let p1 = self.fz.x_otv * root_k;
                let p2 = (center + root_k * (MAX_X + 1.)) as i32;

                let x2 = center - p1;
                root.draw(&Rectangle::new(
                    [(0, 0), (x2 as i32, p2)],
                    ShapeStyle::from(BLACK.filled()),
                ))
                .unwrap();

                let x1 = center + p1;
                root.draw(&Rectangle::new(
                    [(x1 as i32 + 2, 0), (p2, p2)],
                    ShapeStyle::from(BLACK.filled()),
                ))
                .unwrap();
            }
        }

        root.present().unwrap();
    }

    fn draw_bottom_plot2(
        &mut self,
        ui: &mut Ui,
        line: Vec<[f64; 2]>,
        red_point: [f64; 2],
        stud_points: Option<Vec<(f64, f64)>>,
    ) {
        // name of y axis
        let nm = match stud_points {
            Some(_) => "|F|",
            None => "φ",
        };

        let k = self.fz.k();
        // mess when hover on plot near mouse
        let label_fmt = |_s: &str, val: &PlotPoint| {
            format!("u:  {:.3}\nx:  {:.3}\n{nm}: {:.3}", val.x, val.x / k, val.y)
        };

        let x_formatter =
            |mark: GridMark, _range: &RangeInclusive<f64>| format!("{:.2}", mark.value / k);

        let x_axis = vec![
            AxisHints::new_x().label("u"),
            AxisHints::new_x()
                .label("x")
                .formatter(x_formatter)
                .placement(egui_plot::VPlacement::Top),
        ];

        let sz = ui.available_size();
        let x = match stud_points {
            Some(_) => 0.,
            None => sz.x,
        };
        let rect = Rect::from_min_size(Pos2::new(x, sz.x), sz);

        // function for zooming
        let scroll = ui.input(|i| {
            if let Some(mut position) = i.pointer.latest_pos() {
                position.y -= OFFEST_Y;
                position.x -= OFFEST_X;
                if !rect.contains(position) {
                    return None;
                }
            } else {
                return None;
            }
            // if !response.hovered() {
            //     return None;
            // }
            let scroll = i.events.iter().find_map(|e| match e {
                Event::MouseWheel {
                    unit: _,
                    delta,
                    modifiers: _,
                } => Some(*delta),
                _ => None,
            });
            scroll
        });

        Plot::new("bottom_plot".to_owned() + nm)
            .allow_zoom(false)
            .allow_scroll(false)
            .x_grid_spacer(Self::x_grid)
            .y_axis_label(nm)
            .custom_x_axes(x_axis)
            .label_formatter(label_fmt)
            .show(ui, |plot_ui| {
                //add scroll zoom
                if let Some(mut scroll) = scroll {
                    scroll = Vec2::splat(scroll.x + scroll.y);

                    scroll.x /= 10.0;
                    scroll.x = scroll.x.exp(); // i don't know why

                    scroll.y /= 10.0;
                    scroll.y = scroll.y.exp(); // ???

                    plot_ui.zoom_bounds_around_hovered(scroll);
                }
                plot_ui.line(Line::new("u", line).color(Color32::BLUE));

                let points = Points::new("red_point".to_owned() + nm, vec![red_point])
                    .filled(true)
                    .radius(5.)
                    .shape(MarkerShape::Circle)
                    .color(COLOR_RED_POINT_EGUI);
                plot_ui.points(points); // current point

                if let Some(points) = stud_points && *self.fz.get_max_i() != 0.{
                    let points_cross = points
                        .iter()
                        .filter_map(|&(x, i)| {
                            if x == 0. && i == 0. {
                                return None;
                            }
                            let u = x * self.fz.k();
                            Some(self.fz.get_point_norm(u))
                        })
                        .collect::<Vec<_>>();
                    let tmp = Points::new("cross", points_cross)
                        .radius(5.)
                        .shape(MarkerShape::Cross)
                        .color(Color32::RED);
                    plot_ui.points(tmp); // math point

                    let line = points
                        .into_iter()
                        .filter_map(|(x, i)| {
                            if x == 0. && i == 0. {
                                return None;
                            }
                            let u = x * self.fz.k();
                            Some([u, i / *self.fz.get_max_i()])
                        })
                        .collect::<Vec<_>>();

                    plot_ui.line(Line::new("I/Imax", line).color(Color32::ORANGE)); // students points
                }
            })
            .response;
    }

    fn table_ui(&mut self, ui: &mut Ui) {
        use egui_extras::{Column, TableBuilder};
        ui.vertical(|ui| {
            //over table
            let drag = DragValue::new(self.fz.get_max_i()).suffix("мА").speed(0.1);
            add_param(ui, "I без экранов:", drag);

            ui.separator();

            let available_height = ui.available_height();
            let mut table = TableBuilder::new(ui) //table sittings
                .striped(true)
                .resizable(false) // disable resizable
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center)) // place on center
                .column(Column::auto()) // auto size of column
                .column(Column::remainder()) // max size of column
                .column(Column::auto())
                .column(Column::remainder())
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height);

            table = table.sense(egui::Sense::click());

            table
                .header(20.0, |mut header| { // header columns
                    header.col(|ui| {
                        ui.strong("x (см)");
                    });
                    header.col(|ui| {
                        ui.strong("u");
                    });
                    header.col(|ui| {
                        ui.strong("I (мА)");
                    });
                    header.col(|ui| {
                        ui.strong("|F|");
                    });
                })
                .body(|mut body| { // main columns
                    let k = self.fz.k();
                    let mut edited1 = false; // flag for sort student points
                    {
                        let mut points = self.fz.get_student_points();

                        if let Some(&(x, i)) = points.last() {
                            if (x != 0. || i != 0.) && points.len() < 50 { // create new point if last is not 0,0
                                points.push((0., 0.));
                            }
                        } else { //create if students points is empty
                            points.push((0., 0.));
                        }
                        if let Some(i) = points.iter().position(|&point| point == (0., 0.)) {
                            if i < points.len() - 1 { // del the 0,0 point if it not last
                                points.remove(i);
                            }
                        }

                        // draw main table
                        for (row_index, (x, i)) in points.iter_mut().enumerate() {
                            body.row(18., |mut row| {
                                row.set_overline((row_index) % 5 == 0); // separator for every 5

                                const ZERS: f64 = 1000.;
                                let u = ((k * *x) * ZERS).round() / ZERS;
                                //x
                                row.col(|ui| {
                                    let tmp = cell_input(ui, x); // draw cell for input
                                    if tmp && !edited1 {
                                        edited1 = true;
                                    }
                                });
                                //u
                                row.col(|ui| {
                                    ui.label(format!("{u:.2}"));
                                });
                                //I
                                row.col(|ui| {
                                    cell_input(ui, i);
                                });
                                //|F|
                                row.col(|ui| {
                                    // let f = *i / max_i;
                                    // let f = (f * ZERS).round() / ZERS;
                                    // ui.label(f.to_string());

                                    let f = self.fz.get_point_norm(u)[1];
                                    ui.label(format!("{f:.3}"));
                                });
                            });
                        }
                        if edited1 { // sorting
                            points.sort_by(|(x1, _), (x2, _)| x1.total_cmp(x2));
                        }
                    }
                });
        });
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {

                    let available = ui.available_size();
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        // Drow 3d plot
                        let size = Vec2::splat(available.x * 4. / COLS);
                        let inner_ui = &mut alloc_ui_block(ui, size);

                        self.draw_3d(inner_ui, self.fz.difs_3d.clone());

                        // draw params, vawe, ruprs
                        ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                            // draw params, vawe
                            let y1 = ui.available_height();
                            self.parameters(ui);

                            let diff = y1 - ui.available_height();
                            let size = Vec2::splat(size.x - diff);
                            let inner_ui = &mut alloc_ui_block(ui, size);
                            self.draw_wave(inner_ui);
                        });
                    });
                    // draw |F|(u), phi(u)
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        let size = Vec2::new(available.x * 4. / COLS, available.y * 2. / ROWS);

                        // draw |F|(u)
                        let inner_ui = &mut alloc_ui_block(ui, size);
                        let point = self.fz.get_current_point_norm();
                        let line = self.fz.difs.iter().map(|d| d.p_norm()).collect();
                        let tmp = { self.fz.get_student_points().clone() };
                        self.draw_bottom_plot2(inner_ui, line, point, Some(tmp));
                        // self.draw_bottom_plot(inner_ui, line, point, Some(tmp));

                        // draw phi(u)
                        let inner_ui = &mut alloc_ui_block(ui, size);
                        let point = self.fz.get_current_point_arg();
                        let line = self.fz.difs.iter().map(|d| d.p_arg()).collect();
                        self.draw_bottom_plot2(inner_ui, line, point, None);
                    });
                });

                //draw table with students points
                ui.separator();
                let body_text_size = egui::TextStyle::Body.resolve(ui.style()).size;
                use egui_extras::{Size, StripBuilder};
                StripBuilder::new(ui)
                    .size(Size::remainder().at_least(100.0)) // for the table
                    .size(Size::exact(body_text_size)) // for the source code link
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            self.table_ui(ui);
                            // egui::ScrollArea::horizontal().show(ui, |ui| {
                            // });
                        });
                    });
            });
        });

        // ctx.show_viewport_immediate(
        //     egui::ViewportId("dop".into()),
        //     egui::ViewportBuilder::default().with_title("color"),
        //     |ctx, _class| egui::CentralPanel::default().show(ctx, |ui| {
        //     }),
        // );
    }
}

#[inline]
pub fn add_param(ui: &mut Ui, name: &str, drg: DragValue) -> bool {
    let tmp = ui
        .horizontal(|ui| {
            ui.label(name);
            ui.add(drg)
        })
        .inner;
    tmp.changed()
}

#[inline]
fn cell_input(ui: &mut Ui, v: &mut f64) -> bool {
    let copy = *v;
    let resp = ui.add(DragValue::new(v).update_while_editing(false));
    if resp.dragged() {
        *v = copy
    }
    resp.changed()
}

// old code
impl MainApp {
    // fn draw_bottom_plot(
    //     &mut self,
    //     ui: &mut Ui,
    //     line: Vec<(f64, f64)>,
    //     red_point: (f64, f64),
    //     stud_points: Option<Vec<(f64, f64)>>,
    // ) {
    //     let root = EguiBackend::new(ui).into_drawing_area();
    //
    //     if ui.visuals().dark_mode {
    //         root.fill(&BG_PLOT_COLOR_DARK).unwrap();
    //     } else {
    //         root.fill(&BG_PLOT_COLOR_LIGHT).unwrap();
    //     }
    //
    //     const ZERS: f64 = 10.;
    //     let st = if let Some(r) = line.first() {
    //         (r.0 * ZERS).floor() / ZERS
    //     } else {
    //         return;
    //     };
    //     let end = (line.last().unwrap().0 * ZERS).ceil() / ZERS;
    //     let x_axis = (st..(end + 0.01)).step((end - st) / 8.);
    //
    //     let mut min_y = 10.;
    //     let mut max_y = 0.;
    //     for l in line.iter() {
    //         let y = l.1;
    //         if y > max_y {
    //             max_y = y
    //         }
    //         if y < min_y {
    //             min_y = y
    //         }
    //     }
    //
    //     max_y = (max_y * 10.).ceil() / 10.;
    //     min_y = (min_y * 10.).floor() / 10.;
    //
    //     let y_axis = (min_y..max_y).step(0.1);
    //
    //     let mut chart = ChartBuilder::on(&root)
    //         .margin_right(20)
    //         .margin_top(20)
    //         .x_label_area_size(35)
    //         .y_label_area_size(45)
    //         .build_cartesian_2d(x_axis, y_axis)
    //         .unwrap();
    //
    //     let nm = match stud_points {
    //         Some(_) => "|F|",
    //         None => "φ",
    //     };
    //     chart
    //         .configure_mesh()
    //         // .x_desc("u")
    //         .y_desc(nm)
    //         .draw()
    //         .unwrap();
    //
    //     chart
    //         .draw_series(LineSeries::new(line, COLOR_LINE))
    //         .unwrap();
    //
    //     if let Some(points) = stud_points {
    //         // chart
    //         //     .draw_series(points.iter().filter_map(|(x, i)| {
    //         //         if *x == 0. && *i == 0. {
    //         //             return None;
    //         //         }
    //         //         let u = x * self.fz.k();
    //         //         Some(Cross::new(self.fz.get_point_norm(u), 3, &RED))
    //         //     }))
    //         //     .unwrap();
    //
    //         let p = points.into_iter().filter_map(|(x, i)| {
    //             if x == 0. && i == 0. {
    //                 return None;
    //             }
    //             let u = x * self.fz.k();
    //             Some((u, i / self.fz.get_max_i()))
    //         });
    //         chart.draw_series(LineSeries::new(p, &ORANGE)).unwrap();
    //     }
    //
    //     if st != 0. {
    //         chart
    //             .draw_series(LineSeries::new([(0., min_y), (0., max_y)], &BLACK))
    //             .unwrap();
    //     }
    //
    //     chart
    //         .draw_series(std::iter::once(Circle::new(
    //             red_point,
    //             5,
    //             COLOR_RED_POINT.filled(),
    //         )))
    //         .unwrap();
    //
    //     root.present().unwrap();
    // }
}
