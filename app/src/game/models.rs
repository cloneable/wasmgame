extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_macro;
extern crate wasm_logger;
extern crate web_sys;

use crate::engine;

use std::convert::Into;
use std::rc::Rc;
use std::result::{Result, Result::Ok};

use wasm_bindgen::JsValue;

use engine::math::Vec4;
use engine::opengl::Context;
use engine::scene::Model;

pub struct Hexatile {
    pub model: Model,
}

impl Hexatile {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        Ok(Hexatile {
            model: Model::new(
                ctx,
                &super::meshes::HEXATILE_VERTICES,
                &super::meshes::HEXATILE_INDICES,
                3,
            )?,
        })
    }
}

impl engine::scene::Drawable for Hexatile {
    fn init(&mut self, camera: &engine::scene::Camera) {
        self.model[0].translate([-0.6, 0.0, 0.0].into());
        self.model[0].scale([1.0, 3.0, 1.0].into());
        self.model[0].color(Vec4::rgb(0x19, 0x19, 0x70)); // midnightblue
        self.model[0].refresh(camera.view_matrix());

        self.model[1].scale([1.0, 2.0, 1.0].into());
        self.model[1].color(Vec4::rgb(0x87, 0xce, 0xfa)); // lightskyblue
        self.model[1].refresh(camera.view_matrix());

        self.model[2].translate([0.6, 0.0, 0.0].into());
        self.model[2].rotate([0.0, 20.0, 0.0].into());
        self.model[2].color(Vec4::rgb(0xff, 0xb6, 0xc1)); // lightpink
        self.model[2].refresh(camera.view_matrix());

        self.model.init();
        self.model.refresh();
    }

    fn update(&mut self, camera: &engine::scene::Camera) {
        self.model[0].rotate([0.0, 1.0, 0.0].into());
        self.model[0].refresh(camera.view_matrix());
        self.model[1].rotate([0.0, 2.0, 0.0].into());
        self.model[1].refresh(camera.view_matrix());
        self.model[2].rotate([0.0, 3.0, 0.0].into());
        self.model[2].refresh(camera.view_matrix());
        self.model.refresh();
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
