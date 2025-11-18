use crate::windows::math::calc::{converter_freq, DifrPoint};
use fresnel::fresnl;
use std::cell::{RefCell, RefMut};
use std::f64::consts::SQRT_2;

pub const MAX_X: f32 = 20.;

#[derive(PartialEq, Copy, Clone)]
pub enum Screens {
    One,
    Two,
}

pub struct Difr {
    pub x_otv: f32,
    pub l1: f32,
    pub l2: f32,
    pub lambda: f32,
    pub freq: f32,
    pub rezhim: Screens,

    pub difs: Vec<DifrPoint>,
    pub difs_3d: Vec<(f64, f64, f64)>,

    x_otv_c: f32,
    l1_c: f32,
    l2_c: f32,
    lambda_c: f32,

    // pub(crate) max_abs: f64,
    student_points_1: RefCell<Vec<(f64, f64)>>,
    student_points_2: RefCell<Vec<(f64, f64)>>,
    max_i_1: f64,
    max_i_2: f64,
}

impl Difr {
    #[inline]
    pub fn rebuild_integrals(&mut self) {
        const STEP: f64 = 40. / 1000.;
        let k = self.k();

        let us = 0..=(40. / STEP).ceil() as i32;

        self.difs_3d = us
            .clone()
            .map(|x| {
                let x = -MAX_X as f64 + (x as f64) * STEP;
                let u = x * k;

                let (c, s) = fresnl(u);
                (u, c, s)
            })
            .collect();

        let points: Vec<(f64, f64, f64)> = match self.rezhim {
            Screens::One => us
                .map(|x| {
                    let x = -MAX_X as f64 + (x as f64) * STEP;
                    let u = x * k;

                    let (c, s) = fresnl(u);

                    let re = c + 0.5;
                    let im = s + 0.5;
                    let abs = re.hypot(im);
                    let phi = im.atan2(re);
                    (u, abs, phi)
                })
                .collect(),
            Screens::Two => us
                .map(|x| {
                    let x = (x as f64) * STEP / 2.;
                    let u = x * k;

                    let (c, s) = fresnl(u);

                    let re = 2. * c;
                    let im = 2. * s;
                    let abs = re.hypot(im);
                    let phi = im.atan2(re);
                    (u, abs, phi)
                })
                .collect(),
        };

        self.difs = points
            .into_iter()
            .map(|(u, abs, phi)| DifrPoint::new(u, abs / SQRT_2, phi))
            .collect();
    }

    #[inline]
    pub fn get_start(&self) -> f32 {
        match self.rezhim {
            Screens::One => -MAX_X,
            Screens::Two => 0.,
        }
    }

    #[inline]
    pub fn get_fresnel_zones(&self, is_circle: bool) -> Vec<f32> {
        let b = self.b();

        // let max_wave = match self.rezhim {
        //     Screens::One => MAX_X * std::f32::consts::SQRT_2,
        //     Screens::Two => (self.x_otv * self.x_otv * 0.25 + MAX_X * MAX_X).sqrt(),
        // };
        let max_wave = MAX_X * std::f32::consts::SQRT_2;

        let last_n = 2 * (MAX_X * MAX_X / b).round() as i32;

        let mut ret = Vec::new();
        for n in 0..=last_n {
            let r = ((n as f32) * b).sqrt();
            ret.insert(0, r);
            if is_circle && r >= self.x_otv {
                ret[0] = self.x_otv;
                break;
            }
            if r > max_wave {
                break;
            }
        }
        ret
    }

    #[inline]
    pub fn is_cheng(&self) -> bool {
        self.x_otv != self.x_otv_c
            || self.l1 != self.l1_c
            || self.l2 != self.l2_c
            || self.lambda != self.lambda_c
    }

    #[inline]
    pub fn not_sale_cpu_usage(&self) -> bool {
        self.b() > 6.
    }

    #[inline]
    pub fn cheng_copes(&mut self) {
        self.x_otv_c = self.x_otv;
        self.l1_c = self.l1;
        self.l2_c = self.l2;
        self.lambda_c = self.lambda;
    }

    #[inline]
    pub fn backup_copes(&mut self) {
        self.x_otv = self.x_otv_c;
        self.l1 = self.l1_c;
        self.l2 = self.l2_c;
        self.lambda = self.lambda_c;
    }

    #[inline]
    pub fn get_current_point_3d(&self) -> (f64, f64, f64) {
        let u = self.cur_u();

        let (c, s) = fresnl(u);
        (u, c, s)
    }

    #[inline]
    pub fn get_current_points_3d(&self) -> ((f64, f64, f64), (f64, f64, f64)) {
        let u = self.cur_u();

        let (c, s) = fresnl(u);
        ((u, c, s), (-u, -c, -s))
    }

    #[inline]
    pub fn get_current_point_norm(&self) -> [f64; 2] {
        let u = self.cur_u();
        self.get_point_norm(u)
    }

    #[inline]
    pub fn get_point_norm(&self, u: f64) -> [f64; 2] {
        match self.rezhim {
            Screens::One => {
                let (c, s) = fresnl(u);
                [u, (s + 0.5).hypot(c + 0.5) / SQRT_2]
            }
            Screens::Two => {
                let (c, s) = fresnl(u);
                [u, (s * 2.).hypot(c * 2.) / SQRT_2]
            }
        }
    }

    #[inline]
    pub fn get_current_point_arg(&self) -> [f64; 2] {
        let u = self.cur_u();
        match self.rezhim {
            Screens::One => {
                let (c, s) = fresnl(u);
                [u, (s + 0.5).atan2(c + 0.5)]
            }
            Screens::Two => {
                let (c, s) = fresnl(u );
                [u, (s * 2.).atan2(c * 2.)]
            }
        }
    }

    #[inline]
    pub fn k(&self) -> f64 {
        (2. * (self.l1 + self.l2) / (self.lambda * self.l1 * self.l2)).sqrt() as f64
    }

    #[inline]
    fn b(&self) -> f32 {
        (self.lambda * self.l1 * self.l2) / (self.l1 + self.l2)
    }

    #[inline]
    fn cur_u(&self) -> f64 {
        self.k() * self.x_otv as f64
    }

    #[inline]
    pub fn get_student_points(&'_ self) -> RefMut<'_, Vec<(f64, f64)>> {
        match self.rezhim {
            Screens::One => self.student_points_1.borrow_mut(),
            Screens::Two => self.student_points_2.borrow_mut(),
        }
    }

    #[inline]
    pub fn get_max_i(&mut self) -> &mut f64 {
        match self.rezhim {
            Screens::One => &mut self.max_i_1,
            Screens::Two => &mut self.max_i_2,
        }
    }

    // pub fn update_max_i(&mut self) {
    //     let (max_i, points) = match self.rezhim {
    //         Screens::One => (&mut self.max_i_1, self.student_points_1.borrow()),
    //         Screens::Two => (&mut self.max_i_2, self.student_points_2.borrow()),
    //     };
    //
    //     *max_i = points
    //         .iter()
    //         .max_by(|(_, i1), (_, i2)| (*i1).total_cmp(i2))
    //         .unwrap_or(&(0., 0.))
    //         .1;
    // }
}

impl Default for Difr {
    fn default() -> Self {
        let lambda = 3.;

        Self {
            x_otv: 10.,
            l1: 40.,
            l2: 40.,
            lambda,
            freq: converter_freq(lambda),
            rezhim: Screens::One,

            difs: Vec::new(),
            difs_3d: Vec::new(),

            x_otv_c: 0.,
            l1_c: 0.,
            l2_c: 0.,
            lambda_c: 0.,

            // max_abs: 0.,
            student_points_1: RefCell::new(Vec::new()),
            student_points_2: RefCell::new(Vec::new()),
            max_i_1: 0.,
            max_i_2: 0.,
        }
    }
}
