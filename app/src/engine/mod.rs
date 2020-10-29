pub mod core;
pub mod ecs;
pub mod picker;
pub mod scene;
pub mod time;
pub mod util;

use ::std::convert::From;
use ::std::convert::Into;
use ::std::result::Result;
use ::std::string::String;
use ::std::string::ToString;

use ::wasm_bindgen::JsValue;

use time::Time;

pub trait Bindable {
    fn bind(&mut self);
    fn unbind(&mut self);
}

pub trait Drawable {
    fn init(&mut self) -> Result<(), Error>;
    fn update(&mut self, t: Time) -> Result<(), Error>;
    fn draw(&mut self) -> Result<(), Error>;
}

pub mod attrib {
    use crate::util::opengl::Attribute;

    pub const POSITION: Attribute = Attribute(0, 1);
    pub const NORMAL: Attribute = Attribute(1, 1);
    pub const INSTANCE_COLOR: Attribute = Attribute(2, 1);
    pub const INSTANCE_ID: Attribute = Attribute(3, 1);
    pub const MODEL: Attribute = Attribute(4, 4);
    pub const NORMALS: Attribute = Attribute(8, 4);
}

pub enum Error {
    Internal(String),
    JsValue(JsValue),
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error::Internal(msg.to_string())
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Error {
        Error::JsValue(value)
    }
}

impl Into<JsValue> for Error {
    fn into(self) -> JsValue {
        match self {
            Error::Internal(msg) => JsValue::from(msg),
            Error::JsValue(v) => v,
        }
    }
}

impl ::std::fmt::Debug for Error {
    fn fmt(
        &self, f: &mut ::std::fmt::Formatter<'_>,
    ) -> Result<(), ::std::fmt::Error> {
        match self {
            Error::Internal(msg) => f.write_str(msg),
            Error::JsValue(v) => f.write_str(v.as_string().unwrap().as_str()),
        }
    }
}
