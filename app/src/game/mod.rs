mod assets;
mod components;
mod models;
mod shaders;
mod systems;

use ::std::{
    clone::Clone,
    default::Default,
    option::{
        Option,
        Option::{None, Some},
    },
    rc::Rc,
    result::{Result, Result::Ok},
};

use ::wasm_bindgen::JsCast;

use crate::{
    engine, engine::ecs, engine::scene::Camera, engine::time::Time,
    engine::Bindable, engine::Error, util::event, util::math::Vec3,
    util::opengl::Context,
};

struct Scene {
    hexatile_triplet: models::HexatileTriplet,
    camera: Camera,
}

impl Scene {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        let mut camera = Camera::new();
        camera
            .set_position(0.5, 1.0, 3.0)
            .set_target(0.0, 0.0, 0.0)
            .set_frustum(
                60.0,
                ctx.width() as f32 / ctx.height() as f32,
                0.1,
                100.0,
            );
        camera.update(Time::from_millis(0.0));
        let hexatile_triplet = models::HexatileTriplet::new(ctx)?;
        Ok(Scene {
            hexatile_triplet,
            camera,
        })
    }
}

pub struct Game {
    ctx: Rc<Context>,
    world: ecs::World,
    update_runner: ecs::Runner,
    draw_runner: ecs::Runner,

    last_render: Time,
    scene: Scene,
    offscreen: engine::util::OffscreenBuffer,

    picker_shader: engine::picker::PickerShader,
    material_shader: shaders::MaterialShader,

    mouse_down: Option<(i32, i32)>,
    touch_id: Option<i32>,
    camera_position: Vec3,
}

impl Game {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        let mut world = ecs::World::new();
        world.register_component::<components::Spatial>();
        world.register_component::<components::ModelMatrix>();
        world.register_component::<components::HexatileField>();
        world.register_component::<components::Hexatile>();
        world.register_component::<Camera>();
        world.register_component::<components::Timestamp>();
        let mut update_runner = ecs::Runner::new();
        update_runner.register_system(systems::HexatileSystem {
            hexatile_scale: 1.0,
            hexatile_margin: 0.0,
        });
        update_runner.register_system(systems::TransformationSystem);
        let draw_runner = ecs::Runner::new();

        world.set_global(components::Timestamp::default());
        world.set_global(Camera::new());
        let entity1 = world.add_entity();
        world.set_component(entity1, components::Spatial::default());
        world.set_component(entity1, components::ModelMatrix::default());

        let scene = Scene::new(ctx)?;

        let mut picker_shader = engine::picker::PickerShader::new(ctx)?;
        picker_shader.activate();
        picker_shader.set_view(scene.camera.view_matrix());
        picker_shader.set_projection(scene.camera.projection_matrix());

        let mut material_shader = shaders::MaterialShader::new(ctx)?;
        material_shader.activate();
        material_shader.set_view(scene.camera.view_matrix());
        material_shader.set_projection(scene.camera.projection_matrix());

        let camera_position = scene.camera.position();

        Ok(Self {
            ctx: ctx.clone(),
            world,
            update_runner,
            draw_runner,
            last_render: Time::from_millis(0.0),
            scene,
            offscreen: engine::util::OffscreenBuffer::new(
                ctx,
                ctx.width(),
                ctx.height(),
            )?,
            picker_shader,
            material_shader,
            mouse_down: None,
            touch_id: None,
            camera_position,
        })
    }

    pub fn on_click(&mut self, e: &::web_sys::MouseEvent) {
        let (left, top, _, height) = target_rect(e);
        let x = e.client_x() - left as i32;
        let y = e.client_y() - top as i32;
        let rgba = self.offscreen.read_pixel(x, height - y).unwrap();
        ::log::trace!(
            "Clicked at {:?}: {},{}; rgba = {} {} {} {}",
            e.time_stamp(),
            x,
            y,
            rgba[0],
            rgba[1],
            rgba[2],
            rgba[3]
        );
    }

    pub fn on_resize(&mut self, e: &::web_sys::UiEvent) {
        self.ctx.resized();
        ::log::trace!(
            "RESIZE at {:?}: {},{}",
            e.time_stamp(),
            self.ctx.width(),
            self.ctx.height()
        );
    }

    pub fn on_mousedown(&mut self, e: &::web_sys::MouseEvent) {
        let (left, top, ..) = target_rect(e);
        let x = e.client_x() - left;
        let y = e.client_y() - top;
        self.mouse_down = Some((x, y));
        self.camera_position = self.scene.camera.position();
        ::log::trace!("DOWN at {:?}: {},{}", e.time_stamp(), x, y);
    }

    pub fn on_mouseup(&mut self, e: &::web_sys::MouseEvent) {
        let (left, top, ..) = target_rect(e);
        let x = e.client_x() - left;
        let y = e.client_y() - top;
        self.mouse_down = None;
        ::log::trace!("UP at {:?}: {},{}", e.time_stamp(), x, y);
    }

    pub fn on_mousemove(&mut self, e: &::web_sys::MouseEvent) {
        if let Some((ox, oy)) = self.mouse_down {
            let (left, top, ..) = target_rect(e);
            let x = e.client_x() - left;
            let y = e.client_y() - top;
            let new_x = self.camera_position.x + (x - ox) as f32 / 100.0;
            let new_z = self.camera_position.z + (-y + oy) as f32 / 100.0;
            self.scene.camera.set_position(
                new_x,
                self.camera_position.y,
                new_z,
            );
            ::log::trace!("MOVE at {:?}: {},{}", e.time_stamp(), x, y);
        }
    }

    pub fn on_touchstart(&mut self, e: &::web_sys::TouchEvent) {
        ::log::trace!("TOUCH START: {:?}", event::TouchEventWrapper::wrap(e));
        let touch_list = e.target_touches();
        if touch_list.length() != 1 {
            return;
        }
        if let Some(touch) = touch_list.item(0) {
            let (left, top, ..) = target_rect(e);
            let x = touch.client_x() - left;
            let y = touch.client_y() - top;
            self.mouse_down = Some((x, y));
            self.camera_position = self.scene.camera.position();
            self.touch_id = Some(touch.identifier());
            let tew = event::TouchEventWrapper::wrap(e);
            ::log::trace!(
                "TOUCH START at {:?} ({} {}): {:?}",
                e.time_stamp(),
                x,
                y,
                tew
            );
        }
    }

    pub fn on_touchmove(&mut self, e: &::web_sys::TouchEvent) {
        ::log::trace!("TOUCH MOVE: {:?}", event::TouchEventWrapper::wrap(e));
        let touch_list = e.changed_touches();
        for i in 0..touch_list.length() {
            if let Some(touch) = touch_list.item(i) {
                if Some(touch.identifier()) != self.touch_id {
                    return;
                }
                let (left, top, ..) = target_rect(e);
                let x = touch.client_x() - left;
                let y = touch.client_y() - top;
                let (ox, oy) = self.mouse_down.unwrap();
                let new_x = self.camera_position.x + (x - ox) as f32 / 100.0;
                let new_z = self.camera_position.z + (-y + oy) as f32 / 100.0;
                self.scene.camera.set_position(
                    new_x,
                    self.camera_position.y,
                    new_z,
                );
            }
        }
    }

    pub fn on_touchend(&mut self, e: &::web_sys::TouchEvent) {
        ::log::trace!("TOUCH END: {:?}", event::TouchEventWrapper::wrap(e));
        let touch_list = e.changed_touches();
        for i in 0..touch_list.length() {
            if let Some(touch) = touch_list.item(i) {
                if Some(touch.identifier()) != self.touch_id {
                    return;
                }
                let (left, top, ..) = target_rect(e);
                let x = touch.client_x() - left;
                let y = touch.client_y() - top;
                let (ox, oy) = self.mouse_down.unwrap();
                let new_x = self.camera_position.x + (x - ox) as f32 / 100.0;
                let new_z = self.camera_position.z + (-y + oy) as f32 / 100.0;
                self.scene.camera.set_position(
                    new_x,
                    self.camera_position.y,
                    new_z,
                );
                self.mouse_down = None;
                self.touch_id = None;
                ::log::debug!(
                    "TOUCH END at {:?}: #{} {},{}",
                    e.time_stamp(),
                    touch.identifier(),
                    x,
                    y
                );
            }
        }
    }

    pub fn on_touchcancel(&mut self, e: &::web_sys::TouchEvent) {
        ::log::trace!("TOUCH CANCEL: {:?}", event::TouchEventWrapper::wrap(e));
        let touch_list = e.changed_touches();
        for i in 0..touch_list.length() {
            if let Some(touch) = touch_list.item(i) {
                if Some(touch.identifier()) != self.touch_id {
                    return;
                }
                let (left, top, ..) = target_rect(e);
                let x = touch.client_x() - left;
                let y = touch.client_y() - top;
                let (ox, oy) = self.mouse_down.unwrap();
                let new_x = self.camera_position.x + (x - ox) as f32 / 100.0;
                let new_z = self.camera_position.z + (-y + oy) as f32 / 100.0;
                self.scene.camera.set_position(
                    new_x,
                    self.camera_position.y,
                    new_z,
                );
                self.mouse_down = None;
                self.touch_id = None;
                ::log::debug!(
                    "TOUCH CANCEL at {:?}: #{} {},{}",
                    e.time_stamp(),
                    touch.identifier(),
                    x,
                    y
                );
            }
        }
    }

    pub fn on_webglcontextlost(&mut self, e: &::web_sys::WebGlContextEvent) {
        e.prevent_default();
        // TODO: handle context loss.
        // https://www.khronos.org/webgl/wiki/HandlingContextLost
        ::log::warn!("WebGL context lost!");
    }

    pub fn on_webglcontextrestored(
        &mut self, e: &::web_sys::WebGlContextEvent,
    ) {
        e.prevent_default();
        ::log::warn!("WebGL context restored!");
    }

    pub fn on_webglcontextcreationerror(
        &mut self, e: &::web_sys::WebGlContextEvent,
    ) {
        e.prevent_default();
        ::log::warn!("WebGL context creation error!");
    }
}

fn target_rect(e: &::web_sys::Event) -> (i32, i32, i32, i32) {
    let r = e
        .target()
        .unwrap()
        .unchecked_ref::<::web_sys::Element>()
        .get_bounding_client_rect();
    (
        r.left() as i32,
        r.top() as i32,
        r.width() as i32,
        r.height() as i32,
    )
}

impl engine::Drawable for Game {
    fn init(&mut self) -> Result<(), Error> {
        self.scene.hexatile_triplet.init()?;
        self.offscreen.activate();
        Ok(())
    }

    fn update(&mut self, t: Time) -> Result<(), Error> {
        self.world.get_global::<components::Timestamp>().unwrap().t = t;
        self.update_runner.exec(&self.world);

        self.last_render = t;
        self.scene.camera.update(t);
        self.scene.hexatile_triplet.update(t)?;
        self.material_shader.activate();
        self.material_shader
            .set_view(self.scene.camera.view_matrix());
        self.picker_shader.activate();
        self.picker_shader.set_view(self.scene.camera.view_matrix());
        Ok(())
    }

    fn draw(&mut self) -> Result<(), Error> {
        self.draw_runner.exec(&self.world);

        self.offscreen.deactivate();
        self.material_shader.activate();
        self.ctx.gl.clear_color(0.8, 0.7, 0.6, 1.0);
        self.ctx.gl.clear(
            ::web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT
                | ::web_sys::WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        self.scene.hexatile_triplet.bind();
        self.material_shader.activate();
        self.scene.hexatile_triplet.draw()?;

        // for read_pixels.
        self.offscreen.activate();
        self.ctx.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.ctx.gl.clear(
            ::web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT
                | ::web_sys::WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        self.scene.hexatile_triplet.draw()?;
        self.picker_shader.activate();

        self.scene.hexatile_triplet.unbind();

        Ok(())
    }
}

impl engine::core::LoopHandler for Game {
    fn done(&self) -> bool {
        self.last_render >= Time::from_millis(60000.0)
    }
}

impl ::std::ops::Drop for Game {
    fn drop(&mut self) {
        self.offscreen.deactivate();
    }
}
