const _ASSETS_HEXATILE_OBJ: &[u8] = include_bytes!("../../assets/hexatile.obj");
const _ASSETS_HEXATILE_MTL: &[u8] = include_bytes!("../../assets/hexatile.mtl");

::wasmgame_macros::load_objs!(HEXATILE = "app/assets/hexatile.obj");
