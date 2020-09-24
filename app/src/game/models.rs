use ::std::convert::Into;
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};

use ::wasm_bindgen::JsValue;

use crate::engine;
use engine::math::Vec4;
use engine::opengl::Context;
use engine::scene::Model;

pub struct Hexatile {
    pub model: Model,
}

impl Hexatile {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        Ok(Hexatile {
            model: Model::new(ctx, &HEXATILE_VERTICES, &HEXATILE_INDICES, 3)?,
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

const HEX_R: f32 = 0.8660254037844386 * 0.5; //((1.0 - 0.5 * 0.5) as f64).sqrt();

//    c-----b
//   /       \
//  d    y    a
//   \       /
//    e-----f
static HEXATILE_VERTICES: [f32; 3 * (6 + 6)] = [
    // bottom
    0.5, 0.0, 0.0, // 0:a
    0.25, 0.0, -HEX_R, // 1:b
    -0.25, 0.0, -HEX_R, // 2:c
    -0.5, 0.0, 0.0, // 3:d
    -0.25, 0.0, HEX_R, // 4:e
    0.25, 0.0, HEX_R, // 5:f
    // top
    0.5, 0.2, 0.0, // 6:a
    0.25, 0.2, -HEX_R, // 7:b
    -0.25, 0.2, -HEX_R, // 8:c
    -0.5, 0.2, 0.0, // 9:d
    -0.25, 0.2, HEX_R, // 10:e
    0.25, 0.2, HEX_R, // 11:f
];

static HEXATILE_INDICES: [u8; 3 * (4 + 4 + 12)] = [
    // top (CCW fan)
    6, 7, 8, //br
    6, 8, 9, //br
    6, 9, 10, //br
    6, 10, 11, //br
    // bottom (CW fan)
    0, 5, 4, //br
    0, 4, 3, //br
    0, 3, 2, //br
    0, 2, 1, //br
    // sides (strip)
    0, 1, 7, //br
    0, 7, 6, //br
    1, 2, 8, //br
    1, 8, 7, //br
    2, 3, 9, //br
    2, 9, 8, //br
    3, 4, 10, //br
    3, 10, 9, //br
    4, 5, 11, //br
    4, 11, 10, //br
    5, 0, 6, //br
    5, 6, 11, //br
];
