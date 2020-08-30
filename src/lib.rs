use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("wasmgame");

    Ok(())
}
