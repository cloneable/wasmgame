use ::std::option::Option::None;
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};
use ::std::{debug_assert_eq, panic};

use crate::engine::Error;
use crate::util::math::Vec3;
use crate::util::opengl::{Context, Framebuffer, Renderbuffer, Texture2D};

pub fn generate_interleaved_buffer(
    model_indices: &[u8], model_vertices: &[f32], buf: &mut [f32],
) {
    debug_assert_eq!(
        model_indices.len() % 3,
        0,
        "model_indices of wrong length"
    );
    debug_assert_eq!(
        model_vertices.len() % 3,
        0,
        "model_vertices of wrong length"
    );
    debug_assert_eq!(
        buf.len(),
        model_indices.len() * (3 + 3),
        "bad number of vertices"
    );
    let mut i = 0;
    while i < model_indices.len() {
        let ai = model_indices[i] as usize * 3;
        let a = Vec3::with(
            model_vertices[ai],
            model_vertices[ai + 1],
            model_vertices[ai + 2],
        );
        let bi = model_indices[i + 1] as usize * 3;
        let b = Vec3::with(
            model_vertices[bi],
            model_vertices[bi + 1],
            model_vertices[bi + 2],
        );
        let ci = model_indices[i + 2] as usize * 3;
        let c = Vec3::with(
            model_vertices[ci],
            model_vertices[ci + 1],
            model_vertices[ci + 2],
        );

        let j = i * (3 + 3);

        let n = (b - a).cross(c - a).normalize();

        buf[j] = a.x;
        buf[j + 1] = a.y;
        buf[j + 2] = a.z;

        buf[j + 3] = n.x;
        buf[j + 4] = n.y;
        buf[j + 5] = n.z;

        buf[j + 6] = b.x;
        buf[j + 7] = b.y;
        buf[j + 8] = b.z;

        buf[j + 9] = n.x;
        buf[j + 10] = n.y;
        buf[j + 11] = n.z;

        buf[j + 12] = c.x;
        buf[j + 13] = c.y;
        buf[j + 14] = c.z;

        buf[j + 15] = n.x;
        buf[j + 16] = n.y;
        buf[j + 17] = n.z;

        i += 3;
    }
}

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

#[cfg(test)]
pub mod tests {
    use ::std::{assert_eq, panic};
    use ::wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[wasm_bindgen_test]
    fn test_generate_buffers_simple() {
        // 2
        // | \
        // 0--1   4 above 0
        // | /
        // 3
        let model_vertices: [f32; 3 * 5] = [
            0.0, 0.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            1.0, 0.0, -1.0, //br
            1.0, 0.0, 1.0, //br
            0.0, 1.0, 0.0, //br
        ];
        let model_indices: [u8; 3 * 3] = [0, 1, 2, 0, 1, 3, 0, 1, 4];
        let mut buf: [f32; (3 + 3) * 9] = [0.0; (3 + 3) * 9];
        generate_interleaved_buffer(&model_indices, &model_vertices, &mut buf);

        let expect_buf: [f32; (3 + 3) * 9] = [
            0.0, 0.0, 0.0, //br
            0.0, 1.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            0.0, 1.0, 0.0, //br
            1.0, 0.0, -1.0, //br
            0.0, 1.0, 0.0, //br
            //br
            0.0, 0.0, 0.0, //br
            0.0, -1.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            0.0, -1.0, 0.0, //br
            1.0, 0.0, 1.0, //br
            0.0, -1.0, 0.0, //br
            //br
            0.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, //br
            1.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, //br
            0.0, 1.0, 0.0, //br
            0.0, 0.0, 1.0, //br
        ];
        assert_eq!(buf, expect_buf);
    }
}
