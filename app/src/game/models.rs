use ::std::convert::Into;
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};

use super::assets;
use crate::engine;
use crate::engine::time::{Duration, Time};
use crate::util::math::Vec4;
use crate::util::opengl::Context;
use engine::scene::Model;
use engine::Error;

pub struct Hexatile {
    pub model: Model,
}

impl Hexatile {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        Ok(Hexatile {
            model: Model::new(ctx, assets::HEXATILE, 3)?,
        })
    }
}

impl engine::scene::Drawable for Hexatile {
    fn init(&mut self) {
        self.model[0].object.translate([0.0, 0.55, 0.0].into());
        self.model[0].color(Vec4::with_rgb(0x19, 0x19, 0x70)); // midnightblue

        self.model[1].color(Vec4::with_rgb(0x87, 0xce, 0xfa)); // lightskyblue

        self.model[2].object.translate([0.0, -0.55, 0.0].into());
        self.model[2].color(Vec4::with_rgb(0xff, 0xb6, 0xc1)); // lightpink

        self.model.init();
    }

    fn update(&mut self, t: Time) {
        let period = Duration::from_millis(3000.0);
        let offset = Duration::from_millis(500.0);

        let rad = ((t + 0.0 * offset) % period).as_pi(period);
        let deg1 = 3.0 * (rad as f32).sin();
        let rad = ((t + 0.5 * offset) % period).as_pi(period);
        let deg2 = 3.0 * (rad as f32).sin();
        let rad = ((t + 1.0 * offset) % period).as_pi(period);
        let deg3 = 3.0 * (rad as f32).sin();

        self.model[0].object.rotate([0.0, deg1, 0.0].into());
        self.model[1].object.rotate([0.0, deg2, 0.0].into());
        self.model[2].object.rotate([0.0, deg3, 0.0].into());
        self.model.update(t);
    }

    fn stage(&mut self) {
        self.model.select();
    }

    fn draw(&self) {
        self.model.draw();
    }

    fn unstage(&mut self) {
        self.model.unselect();
    }
}
