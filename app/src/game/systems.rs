use crate::{
    engine::{ecs, ecs::Joiner},
    game::{
        components::{Hexatile, ModelMatrix, Spatial, Timestamp},
        Camera,
    },
    util::math::Vec3,
};

// Outer hexagon radius of normalized inner  radius (1.0).
const HEXATILE_OUTER_RADIUS: f32 = 1.118033988749895;

pub struct HexatileSystem {
    // Scaling factor for correction.
    pub hexatile_scale: f32,
    // Gap between tiles as fraction of scaled inner radius.
    pub hexatile_margin: f32,
}

impl HexatileSystem {
    fn calculate_position(&self, u: i16, w: i16) -> Vec3 {
        let (u, w) = (u as f32, w as f32);
        Vec3::with(
            self.hexatile_scale * (HEXATILE_OUTER_RADIUS * 1.5) * u,
            0.0,
            self.hexatile_scale * (w - 0.5 * u),
        )
    }
}

impl<'a> ecs::System<'a> for HexatileSystem {
    type Args = (ecs::PerEntity<'a, Hexatile>, ecs::PerEntity<'a, Spatial>);

    fn exec(&mut self, (hexatile, spatial): Self::Args) {
        for (hex, s) in (&hexatile, &spatial).join() {
            if hex.changed {
                hex.changed = false;
                let (mut u, mut v, mut w) = hex.uvw;
                if v != 0 {
                    u += v;
                    w += v;
                    v = 0;
                    hex.uvw = (u, v, w);
                }
                s.position = self.calculate_position(u, w);
            }
        }
    }
}

pub struct TransformationSystem;

impl<'a> ecs::System<'a> for TransformationSystem {
    type Args = (
        ecs::Global<'a, Camera>,
        ecs::Global<'a, Timestamp>,
        ecs::PerEntity<'a, Spatial>,
        ecs::PerEntity<'a, ModelMatrix>,
    );

    fn exec(&mut self, (mut _camera, _ts, spatial, model): Self::Args) {
        for (s, m) in (&spatial, &model).join() {
            m.model.translate(s.position);
        }
    }
}
