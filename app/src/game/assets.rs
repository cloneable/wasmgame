const _ASSETS_CUBE_OBJ: &[u8] = include_bytes!("../../assets/cube.obj");
const _ASSETS_CUBE_MTL: &[u8] = include_bytes!("../../assets/cube.mtl");
const _ASSETS_HEXATILE_OBJ: &[u8] = include_bytes!("../../assets/hexatile.obj");
const _ASSETS_HEXATILE_MTL: &[u8] = include_bytes!("../../assets/hexatile.mtl");

::wasmgame_macros::load_objs!("app/assets/cube.obj", "app/assets/hexatile.obj");
