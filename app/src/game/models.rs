use ::std::{
    assert_eq,
    convert::Into,
    panic,
    rc::Rc,
    result::{Result, Result::Ok},
    vec::Vec,
};

use crate::{
    engine::{
        scene::{Instance, Mesh},
        time::{Duration, Time},
        Bindable, Drawable, Error,
    },
    game::assets,
    util::{math::Vec4, opengl::Context},
};

// TODO: Use new Mesh type once available.
struct Hexatile {
    mesh: Mesh,
}

impl Hexatile {
    pub fn new(ctx: &Rc<Context>, instances: usize) -> Result<Self, Error> {
        Ok(Hexatile {
            mesh: Mesh::new(ctx, assets::HEXATILE, instances)?,
        })
    }
}

pub struct HexatileTriplet {
    hexatile: Hexatile,
}

impl HexatileTriplet {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        Ok(HexatileTriplet {
            hexatile: Hexatile::new(ctx, 3)?,
        })
    }
}

impl Drawable for HexatileTriplet {
    fn init(&mut self) -> Result<(), Error> {
        self.hexatile.mesh[0]
            .object
            .translate([0.0, 0.55, 0.0].into());
        self.hexatile.mesh[0].color(Vec4::with_rgb(0x19, 0x19, 0x70)); // midnightblue

        self.hexatile.mesh[1].color(Vec4::with_rgb(0x87, 0xce, 0xfa)); // lightskyblue

        self.hexatile.mesh[2]
            .object
            .translate([0.0, -0.55, 0.0].into());
        self.hexatile.mesh[2].color(Vec4::with_rgb(0xff, 0xb6, 0xc1)); // lightpink

        self.hexatile.mesh.init()
    }

    fn update(&mut self, t: Time) -> Result<(), Error> {
        let period = Duration::from_millis(3000.0);
        let offset = Duration::from_millis(500.0);

        let rad = ((t + 0.0 * offset) % period).as_pi(period);
        let deg1 = 3.0 * (rad as f32).sin();
        let rad = ((t + 0.5 * offset) % period).as_pi(period);
        let deg2 = 3.0 * (rad as f32).sin();
        let rad = ((t + 1.0 * offset) % period).as_pi(period);
        let deg3 = 3.0 * (rad as f32).sin();

        self.hexatile.mesh[0].object.rotate([0.0, deg1, 0.0].into());
        self.hexatile.mesh[1].object.rotate([0.0, deg2, 0.0].into());
        self.hexatile.mesh[2].object.rotate([0.0, deg3, 0.0].into());
        self.hexatile.mesh.update(t)
    }

    fn draw(&mut self) -> Result<(), Error> {
        self.hexatile.mesh.draw()
    }
}

impl Bindable for HexatileTriplet {
    fn bind(&mut self) {
        self.hexatile.mesh.bind();
    }

    fn unbind(&mut self) {
        self.hexatile.mesh.unbind();
    }
}
