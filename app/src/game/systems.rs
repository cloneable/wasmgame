use super::components::{Camera, ModelMatrix, Spatial};
use crate::engine::ecs;
use crate::engine::ecs::Joiner;

pub struct TransformationSystem;

impl<'a> ecs::System<'a> for TransformationSystem {
    type Args = (
        ecs::Global<'a, Camera>,
        ecs::PerEntity<'a, Spatial>,
        ecs::PerEntity<'a, ModelMatrix>,
    );

    fn exec(&mut self, (_camera, spatial, model): Self::Args) {
        for (s, m) in (&spatial, &model).join() {
            m.model.translate(s.position);
        }
    }
}
