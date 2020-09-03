use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub trait Renderer {
    fn render(&mut self, timestamp: std::time::Duration) -> Result<(), JsValue>;
    fn ready(&self) -> bool;
    fn done(&self) -> bool;
}

pub fn enter_loop(
    window: Rc<RefCell<web_sys::Window>>,
    renderer: Rc<RefCell<dyn Renderer>>,
) -> Result<(), JsValue> {
    log::info!("render_loop starting");
    let f = Rc::new(RefCell::new(None as Option<Closure<dyn FnMut(f64) + 'static>>));
    let g = f.clone();
    let captured_window = window.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        if renderer.borrow().done() {
            log::info!("render_loop exiting");
            let _ = f.borrow_mut().take();
            return;
        }

        if renderer.borrow().ready() {
            renderer.borrow_mut().render(std::time::Duration::from_micros((timestamp * 1000.0) as u64)).unwrap();
        }

        captured_window
            .borrow()
            .request_animation_frame(
                (f.borrow().as_ref().unwrap() as &Closure<dyn FnMut(f64)>)
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
    }) as Box<dyn FnMut(f64) + 'static>));

    window
        .borrow()
        .request_animation_frame(
            (g.borrow().as_ref().unwrap() as &Closure<dyn FnMut(f64)>)
                .as_ref()
                .unchecked_ref(),
        )
        .unwrap();

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
