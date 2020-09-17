extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use crate::opengl;
use crate::math;

use std::boxed::Box;
use std::cell::RefCell;
use std::clone::Clone;
use std::convert::AsRef;
use std::ops::FnMut;
use std::option::{Option, Option::None, Option::Some};
use std::rc::Rc;
use std::result::{Result, Result::Ok};
use std::{debug_assert_eq, panic};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

pub trait Renderer {
    fn setup(&mut self, ctx: &opengl::Context) -> Result<(), JsValue>;
    fn render(&mut self, ctx: &opengl::Context, millis: f64) -> Result<(), JsValue>;
    fn done(&self) -> bool;
}

type RequestAnimationFrameCallback = Closure<dyn FnMut(f64) + 'static>;

fn request_animation_frame_helper(callback: Option<&RequestAnimationFrameCallback>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(callback.unwrap().as_ref().unchecked_ref())
        .unwrap();
}

pub struct Engine {
    pub ctx: opengl::Context,
    renderer: Rc<RefCell<dyn Renderer>>,
}

impl Engine {
    pub fn new(ctx: opengl::Context, renderer: Rc<RefCell<dyn Renderer>>) -> Rc<Self> {
        Rc::new(Self { ctx, renderer })
    }

    pub fn start(self: Rc<Self>) -> Result<(), JsValue> {
        self.renderer.borrow_mut().setup(&self.ctx)?;
        // Part of this is taken from the wasm-bindgen guide.
        // This kinda works for now, but needs to be checked for
        // leaks.
        // TODO: Check if renderer, callback instances not freed.
        // TODO: See if there's a better/cleaner way to do this.
        let callback = Rc::new(RefCell::new(None as Option<RequestAnimationFrameCallback>));
        let c = callback.clone();
        *callback.borrow_mut() = Some(Closure::wrap(Box::new(move |millis: f64| {
            if self.renderer.borrow().done() {
                let _ = c.borrow_mut().take();
                log::info!("wasmgame ending");
                return;
            }

            self.renderer
                .borrow_mut()
                .render(&self.ctx, millis)
                .unwrap();

            let c0 = c.clone();
            request_animation_frame_helper(c0.borrow().as_ref());
        }) as Box<dyn FnMut(f64) + 'static>));

        request_animation_frame_helper(callback.borrow().as_ref());
        Ok(())
    }
}

pub fn generate_buffers(
    model_indices: &[u8],
    model_vertices: &[f32],
    vertices: &mut [f32],
    normals: &mut [f32],
) {
    debug_assert_eq!(model_indices.len() % 3, 0, "model_indices of wrong length");
    debug_assert_eq!(
        model_vertices.len() % 3,
        0,
        "model_vertices of wrong length"
    );
    debug_assert_eq!(
        vertices.len(),
        model_indices.len() * 3,
        "bad number of vertices"
    );
    debug_assert_eq!(
        normals.len(),
        model_indices.len() * 3,
        "bad number of normals"
    );
    let mut i = 0;
    while i < model_indices.len() {
        let ai = model_indices[i] as usize * 3;
        let a = math::Vec3::new(
            model_vertices[ai + 0],
            model_vertices[ai + 1],
            model_vertices[ai + 2],
        );
        let bi = model_indices[i + 1] as usize * 3;
        let b = math::Vec3::new(
            model_vertices[bi + 0],
            model_vertices[bi + 1],
            model_vertices[bi + 2],
        );
        let ci = model_indices[i + 2] as usize * 3;
        let c = math::Vec3::new(
            model_vertices[ci + 0],
            model_vertices[ci + 1],
            model_vertices[ci + 2],
        );

        let n = (&b - &a).cross(&(&c - &a)).normalize();

        let j = i * 3;
        vertices[j + 0] = a.x;
        vertices[j + 1] = a.y;
        vertices[j + 2] = a.z;
        vertices[j + 3] = b.x;
        vertices[j + 4] = b.y;
        vertices[j + 5] = b.z;
        vertices[j + 6] = c.x;
        vertices[j + 7] = c.y;
        vertices[j + 8] = c.z;

        normals[j + 0] = n.x;
        normals[j + 1] = n.y;
        normals[j + 2] = n.z;
        normals[j + 3] = n.x;
        normals[j + 4] = n.y;
        normals[j + 5] = n.z;
        normals[j + 6] = n.x;
        normals[j + 7] = n.y;
        normals[j + 8] = n.z;

        i += 3;
    }
}

#[cfg(test)]
pub mod tests {
    extern crate std;
    extern crate wasm_bindgen_test;
    use std::{assert_eq, panic};
    use wasm_bindgen_test::wasm_bindgen_test;

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
        let mut vertices: [f32; 3 * 9] = [0.0; 3 * 9];
        let mut normals: [f32; 3 * 9] = [0.0; 3 * 9];
        generate_buffers(&model_indices, &model_vertices, &mut vertices, &mut normals);

        let expect_vertices: [f32; 3 * 9] = [
            0.0, 0.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            1.0, 0.0, -1.0, //br
            0.0, 0.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            1.0, 0.0, 1.0, //br
            0.0, 0.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            0.0, 1.0, 0.0, //br
        ];
        let expect_normals: [f32; 3 * 9] = [
            0.0, 1.0, 0.0, //br
            0.0, 1.0, 0.0, //br
            0.0, 1.0, 0.0, //br
            0.0, -1.0, 0.0, //br
            0.0, -1.0, 0.0, //br
            0.0, -1.0, 0.0, //br
            0.0, 0.0, 1.0, //br
            0.0, 0.0, 1.0, //br
            0.0, 0.0, 1.0, //br
        ];
        assert_eq!(vertices, expect_vertices);
        assert_eq!(normals, expect_normals);
    }
}
