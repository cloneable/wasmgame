extern crate std;

use std::{debug_assert_eq, panic};

use super::math::Vec3;

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
        let a = Vec3::new(
            model_vertices[ai + 0],
            model_vertices[ai + 1],
            model_vertices[ai + 2],
        );
        let bi = model_indices[i + 1] as usize * 3;
        let b = Vec3::new(
            model_vertices[bi + 0],
            model_vertices[bi + 1],
            model_vertices[bi + 2],
        );
        let ci = model_indices[i + 2] as usize * 3;
        let c = Vec3::new(
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
