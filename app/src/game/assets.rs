const _WEIRD_CUBE_OBJ: &[u8] = include_bytes!("../../assets/cube.obj");
const _WEIRD_CUBE_MTL: &[u8] = include_bytes!("../../assets/cube.mtl");

::wasmgame_macros::load_obj!("app/assets/cube.obj");
