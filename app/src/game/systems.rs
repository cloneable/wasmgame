use super::components::{ModelMatrix, Spatial, Timestamp};
use super::Camera;
use crate::engine::ecs;
use crate::engine::ecs::Joiner;

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
