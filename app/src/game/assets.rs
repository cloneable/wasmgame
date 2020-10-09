static WEIRD_CUBE_OBJ: &'static [u8] = include_bytes!("../../assets/cube.obj");
static WEIRD_CUBE_MTL: &'static [u8] = include_bytes!("../../assets/cube.mtl");

static X: &'static str = ::wasmgame_macros::load_obj!("app/assets/cube.obj");
