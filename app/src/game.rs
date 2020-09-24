mod models;
mod shaders;

use ::std::clone::Clone;
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};
use ::std::time::Duration;

use ::wasm_bindgen::JsCast;

use crate::engine;
use crate::util::opengl::Context;
use engine::scene::{Camera, Drawable};
use engine::Error;

struct Scene {
    hexatile: models::Hexatile,
    camera: Camera,
}

impl Scene {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        let mut camera = Camera::new();
        camera
            .set_position(0.5, 1.4, 3.0)
            .set_target(0.0, 0.0, 0.0)
            .set_frustum(35.0, 4.0 / 3.0, 0.1, 100.0)
            .refresh();
        let hexatile = models::Hexatile::new(ctx)?;
        Ok(Scene { hexatile, camera })
    }
}

pub struct Game {
    ctx: Rc<Context>,

    last_render: Duration,
    scene: Scene,
    offscreen: engine::util::OffscreenBuffer,

    picker_program: engine::picker::PickerProgram,
    program: shaders::HexatileProgram,
}

impl Game {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        let scene = Scene::new(ctx)?;

        let mut picker_program = engine::picker::PickerProgram::new(ctx)?;
        picker_program.activate();
        picker_program.set_view(scene.camera.view_matrix());
        picker_program.set_projection(scene.camera.projection_matrix());

        let mut program = shaders::HexatileProgram::new(ctx)?;
        program.activate();
        program.set_view(scene.camera.view_matrix());
        program.set_projection(scene.camera.projection_matrix());

        Ok(Self {
            ctx: ctx.clone(),
            last_render: Duration::from_secs(0),
            scene,
            offscreen: engine::util::OffscreenBuffer::new(ctx, 400, 300)?,
            picker_program,
            program,
        })
    }

    pub fn init(&mut self) -> Result<(), Error> {
        // TODO: refactor why camera is pulled in.
        self.scene.hexatile.init(&self.scene.camera);
        self.offscreen.activate();
        Ok(())
    }
}

impl engine::Renderer for Game {
    fn update(&mut self, timestamp: Duration) -> Result<(), Error> {
        self.last_render = timestamp;
        self.scene.hexatile.update(&self.scene.camera);
        Ok(())
    }

    fn render(&mut self, _timestamp: Duration) -> Result<(), Error> {
        self.offscreen.deactivate();

        // draw

        self.scene.hexatile.stage();

        self.program.activate();
        self.ctx.gl.clear_color(0.8, 0.7, 0.6, 1.0);
        self.ctx.gl.clear(
            ::web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT
                | ::web_sys::WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );
        self.scene.hexatile.draw();

        // for read_pixels.
        self.picker_program.activate();
        self.offscreen.activate();
        self.ctx.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.ctx.gl.clear(
            ::web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT
                | ::web_sys::WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );
        self.scene.hexatile.draw();

        self.scene.hexatile.unstage();

        Ok(())
    }

    fn done(&self) -> bool {
        self.last_render >= Duration::from_secs(10)
    }
}

// TODO: use const generic for event type name.
impl engine::EventHandler<::web_sys::MouseEvent> for Game {
    fn handle(&mut self, timestamp: Duration, event: &::web_sys::MouseEvent) {
        // TODO: Experiment with a #[wasm_bindgen(inline_js) function
        //       that does most calls in JS.
        let r = event
            .target()
            .unwrap()
            .unchecked_ref::<::web_sys::Element>()
            .get_bounding_client_rect();
        let x = event.client_x() - r.left() as i32;
        let y = event.client_y() - r.top() as i32;
        let rgba = self.offscreen.read_pixel(x, r.height() as i32 - y).unwrap();
        ::log::debug!(
            "Clicked at {}: {},{}; rgba = {} {} {} {}",
            timestamp.as_micros(),
            x,
            y,
            rgba[0],
            rgba[1],
            rgba[2],
            rgba[3]
        );
    }
}
