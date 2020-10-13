use ::std::convert::Into;
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};
use ::std::vec::Vec;
use ::std::{assert_eq, panic};

use super::assets;
use crate::engine;
use crate::engine::time::{Duration, Time};
use crate::util::math::Vec4;
use crate::util::opengl::Context;
use engine::scene::Instance;
use engine::scene::Model;
use engine::Error;

// TODO: Use new Mesh type once available.
struct Hexatile {
    model: Model,
}

impl Hexatile {
    pub fn new(ctx: &Rc<Context>, instances: usize) -> Result<Self, Error> {
        Ok(Hexatile {
            model: Model::new(ctx, assets::HEXATILE, instances)?,
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

impl engine::Drawable for HexatileTriplet {
    fn init(&mut self) -> Result<(), Error> {
        self.hexatile.model[0]
            .object
            .translate([0.0, 0.55, 0.0].into());
        self.hexatile.model[0].color(Vec4::with_rgb(0x19, 0x19, 0x70)); // midnightblue

        self.hexatile.model[1].color(Vec4::with_rgb(0x87, 0xce, 0xfa)); // lightskyblue

        self.hexatile.model[2]
            .object
            .translate([0.0, -0.55, 0.0].into());
        self.hexatile.model[2].color(Vec4::with_rgb(0xff, 0xb6, 0xc1)); // lightpink

        self.hexatile.model.init()
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

        self.hexatile.model[0]
            .object
            .rotate([0.0, deg1, 0.0].into());
        self.hexatile.model[1]
            .object
            .rotate([0.0, deg2, 0.0].into());
        self.hexatile.model[2]
            .object
            .rotate([0.0, deg3, 0.0].into());
        self.hexatile.model.update(t)
    }

    fn draw(&mut self) -> Result<(), Error> {
        self.hexatile.model.draw()
    }
}

impl engine::Bindable for HexatileTriplet {
    fn bind(&mut self) {
        self.hexatile.model.bind();
    }

    fn unbind(&mut self) {
        self.hexatile.model.unbind();
    }
}

static ACTIVE_TILES_MAP: [u8; 10 * 10] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //br
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //br
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //br
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //br
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //br
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //br
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //br
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //br
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0, //br
    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, //br
];

pub struct Board {
    width: usize,
    height: usize,
    hexatiles: Hexatile,
    indices: Vec<usize>,
    index_map: Vec<i32>,
}

impl Board {
    pub fn new(
        ctx: &Rc<Context>, active_tiles: &[u8], width: usize,
    ) -> Result<Self, Error> {
        assert_eq!(active_tiles.len() % width, 0);
        let active_tiles: &[u8] = &ACTIVE_TILES_MAP;
        let width: usize = 10;
        let height: usize = active_tiles.len() / width;

        let mut indices: Vec<usize> = Vec::with_capacity(width * height);
        let mut index_map: Vec<i32> = Vec::with_capacity(width * height);
        index_map.resize(width * height, -1);
        for y in 0..height {
            for x in 0..width {
                let offset = y * width + x;
                if active_tiles[offset] != 0 {
                    index_map[offset] = indices.len() as i32;
                    indices.push(offset);
                }
            }
        }

        indices.shrink_to_fit();
        let num_instances = indices.len();
        Ok(Board {
            width,
            height,
            hexatiles: Hexatile::new(ctx, num_instances)?,
            indices,
            index_map,
        })
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn coords(&self, i: usize) -> (usize, usize) {
        let x = i % self.width;
        let y = i / self.width;
        (x, y)
    }

    pub fn hexatile(&mut self, x: usize, y: usize) -> &mut Instance {
        let offset = self.index(x, y);
        &mut self.hexatiles.model[offset]
    }
}

impl engine::Drawable for Board {
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn update(&mut self, _t: Time) -> Result<(), Error> {
        Ok(())
    }

    fn draw(&mut self) -> Result<(), Error> {
        self.hexatiles.model.draw()
    }
}
