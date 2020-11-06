use crate::{
    engine::{ecs, ecs::Joiner},
    game::{
        components::{ModelMatrix, Spatial, Timestamp},
        Camera,
    },
};

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
