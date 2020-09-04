use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub trait Renderer {
    fn prep(&mut self, millis: f64) -> Result<(), JsValue>;
    fn render(&mut self, millis: f64) -> Result<(), JsValue>;
    fn ready(&self) -> bool;
    fn done(&self) -> bool;
}

type RequestAnimationFrameCallback = Closure<dyn FnMut(f64)>;

fn request_animation_frame_helper(callback: Option<&RequestAnimationFrameCallback>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(callback.unwrap().as_ref().unchecked_ref())
        .unwrap();
}

pub fn enter_loop(renderer: Rc<RefCell<dyn Renderer>>) -> Result<(), JsValue> {
    // Part of this is taken from the wasm-bindgen guide.
    // This kinda works for now, but needs to be checked for
    // leaks.
    // TODO: Check if renderer, callback instances not freed.
    // TODO: See if there's a better/cleaner way to do this.
    log::debug!("render_loop starting");
    let callback = Rc::new(RefCell::new(None as Option<RequestAnimationFrameCallback>));
    let c = callback.clone();
    *callback.borrow_mut() = Some(Closure::wrap(Box::new(move |millis: f64| {
        if renderer.borrow().done() {
            log::debug!("render_loop exiting");
            let _ = c.borrow_mut().take();
            return;
        }

        if renderer.borrow().ready() {
            renderer.borrow_mut().render(millis).unwrap();
        }

        let c2 = c.clone();
        let r2 = renderer.clone();
        wasm_bindgen_futures::spawn_local(async move {
            r2.borrow_mut().prep(millis).unwrap();
            request_animation_frame_helper(c2.borrow().as_ref());
        });
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame_helper(callback.borrow().as_ref());
    Ok(())
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum Color {
    RGBA(u8, u8, u8, u8),
    RGB(u8, u8, u8),
    Gray(u8),

    Black,
    White,
}

impl Color {
    fn write_rgba(data: &mut Vec<u8>, offset: usize, r: u8, g: u8, b: u8, a: u8) {
        data[offset] = r;
        data[offset + 1] = g;
        data[offset + 2] = b;
        data[offset + 3] = a;
    }

    pub fn write(self, data: &mut Vec<u8>, offset: usize) {
        match self {
            Color::RGBA(r, g, b, a) => Color::write_rgba(data, offset, r, g, b, a),
            Color::RGB(r, g, b) => Color::write_rgba(data, offset, r, g, b, 255),
            Color::Gray(v) => Color::write_rgba(data, offset, v, v, v, 255),
            Color::Black => Color::write_rgba(data, offset, 0, 0, 0, 255),
            Color::White => Color::write_rgba(data, offset, 255, 255, 255, 255),
        };
    }
}

pub struct Canvas {
    data: Vec<u8>,
    image_data: web_sys::ImageData,
    w: u32,
    h: u32,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self {
        let mut data: Vec<u8> = vec![0; (w * h * 4) as usize];
        let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&mut data),
            w,
            h,
        )
        .unwrap();
        Self {
            data,
            image_data,
            w,
            h,
        }
    }

    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }

    pub fn image_data(&self) -> &web_sys::ImageData {
        &self.image_data
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        let offset = ((y * self.w + x) * 4) as usize;
        color.write(&mut self.data, offset);
    }

    pub fn fill(&mut self, color: Color) {
        let mut offset = 0;
        while offset < self.data.len() {
            color.write(&mut self.data, offset);
            offset += 4;
        }
    }
}

//const HEX_R : f64 = 0.8660254037844386; //((1.0 - 0.5 * 0.5) as f64).sqrt();

pub fn stroke_hexatile(
    off_ctx: &web_sys::CanvasRenderingContext2d,
    x: f64,
    y: f64,
    rx: f64,
    ry: f64,
) {
    //  5  6
    // 4 <>-1
    //  3  2
    off_ctx.begin_path();
    off_ctx.move_to(x + rx, y);
    off_ctx.line_to(x + rx * 0.5, y + ry);
    off_ctx.line_to(x - rx * 0.5, y + ry);
    off_ctx.line_to(x - rx, y);
    off_ctx.line_to(x - rx * 0.5, y - ry);
    off_ctx.line_to(x + rx * 0.5, y - ry);
    off_ctx.close_path();
    off_ctx.stroke();
}
