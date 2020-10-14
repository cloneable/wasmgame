use ::std::option::Option::None;
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};

use crate::engine::Error;
use crate::util::opengl::{Context, Framebuffer, Renderbuffer, Texture2D};

pub struct OffscreenBuffer {
    framebuffer: Framebuffer,
    colorbuffer: Texture2D,
    depthbuffer: Renderbuffer,
}

impl OffscreenBuffer {
    pub fn new(
        ctx: &Rc<Context>, width: i32, height: i32,
    ) -> Result<Self, Error> {
        let mut colorbuffer = Texture2D::create(ctx)?;
        colorbuffer.bind();
        colorbuffer.tex_image_2d(width, height, None)?;
        colorbuffer.unbind();

        let mut depthbuffer = Renderbuffer::create(ctx)?;
        depthbuffer.bind();
        depthbuffer.storage_for_depth(width, height);
        depthbuffer.unbind();

        let mut framebuffer = Framebuffer::create(ctx)?;
        framebuffer.bind();
        framebuffer.texture2d_as_colorbuffer(&colorbuffer);
        framebuffer.renderbuffer_as_depthbuffer(&depthbuffer);
        {
            let status = framebuffer.check_status();
            if status != ::web_sys::WebGl2RenderingContext::FRAMEBUFFER_COMPLETE
            {
                ::log::error!("framebuffer incomplete: {}", status)
            }
        }
        framebuffer.unbind();
        Ok(OffscreenBuffer {
            framebuffer,
            colorbuffer,
            depthbuffer,
        })
    }

    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), Error> {
        self.colorbuffer.bind();
        self.colorbuffer.tex_image_2d(width, height, None)?;
        self.colorbuffer.unbind();
        self.depthbuffer.bind();
        self.depthbuffer.storage_for_depth(width, height);
        self.depthbuffer.unbind();
        Ok(())
    }

    pub fn activate(&mut self) {
        self.framebuffer.bind()
    }

    pub fn deactivate(&mut self) {
        self.framebuffer.unbind()
    }

    pub fn read_pixel(&self, x: i32, y: i32) -> Result<[u8; 4], Error> {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        self.framebuffer.read_pixels(
            x,
            y,
            1,
            1,
            ::web_sys::WebGl2RenderingContext::RGBA,
            ::web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
            &mut buf[..],
        )?;
        Ok(buf)
    }
}
