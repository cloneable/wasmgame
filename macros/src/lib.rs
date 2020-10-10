#![no_implicit_prelude]

use ::std::convert::From;
use ::std::iter::Iterator;
use ::std::option::Option::Some;
use ::std::result::{Result::Err, Result::Ok};
use ::std::vec::Vec;
use ::std::{assert_eq, panic};

use ::obj::{IndexTuple, ObjData, SimplePolygon};
use ::proc_macro::TokenStream;
use ::quote::quote;
use ::syn::parse::{Parse, ParseStream};
use ::syn::{parse_macro_input, LitStr, Result};

struct Input {
    obj_path: LitStr,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            Ok(Input {
                obj_path: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

fn process_input(input: Input) -> Result<::obj::Obj> {
    let path = input.obj_path.value();
    let cwd = ::std::env::current_dir()
        .map_err(|err| ::syn::Error::new(input.obj_path.span(), err))?;
    let mut o: ::obj::Obj = ::obj::Obj::load_with_config(
        cwd.join(path),
        ::obj::LoadConfig { strict: true },
    )
    .map_err(|err| ::syn::Error::new(input.obj_path.span(), err))?;
    o.load_mtls()
        .map_err(|err| ::syn::Error::new(input.obj_path.span(), err))?;
    Ok(o)
}

fn append_vertex(data: &ObjData, v: IndexTuple, points: &mut Vec<f32>) {
    {
        let vertex_index = v.0;
        let vertex = data.position[vertex_index];
        assert_eq!(vertex.len(), 3);
        points.extend_from_slice(&vertex);
    }
    if let Some(normal_index) = v.2 {
        let normal = data.normal[normal_index];
        assert_eq!(normal.len(), 3);
        points.extend_from_slice(&normal);
    } else {
        points.push(0.0);
        points.push(0.0);
        points.push(0.0);
    }
    // TODO: Add texture mapping.
    // if let Some(texture_index) = v.1 {
    //     let texture = data.texture[texture_index];
    //     assert_eq!(texture.len(), 2);
    //     points.extend_from_slice(&texture);
    // } else {
    //     points.push(0.0);
    //     points.push(0.0);
    // }
}

fn append_triangles(data: &ObjData, p: &SimplePolygon, points: &mut Vec<f32>) {
    let index_tuples = &p.0;
    assert!(index_tuples.len() >= 3);
    for i in 1..(index_tuples.len() - 1) {
        append_vertex(data, index_tuples[0], points);
        append_vertex(data, index_tuples[i], points);
        append_vertex(data, index_tuples[i + 1], points);
    }
}

fn generate_output(o: ::obj::Obj) -> TokenStream {
    let data = &o.data;
    let mut points = Vec::<f32>::with_capacity(2000);
    let mut objects_code = Vec::with_capacity(data.objects.len());
    for ob in &data.objects {
        let name = &ob.name;
        let obj_start = points.len();
        for g in &ob.groups {
            for p in &g.polys {
                append_triangles(data, p, &mut points);
            }
        }
        let obj_end = points.len();
        objects_code.push(quote! {
            Object {
                name: #name,
                start: #obj_start,
                end: #obj_end,
            }
        });
    }
    let num_points = points.len();
    assert_eq!(num_points % 6, 0);
    let num_objects = objects_code.len();

    let t = quote! {
        pub struct Object {
            name: &'static str,
            start: usize,
            end: usize,
        }

        impl Object {
            pub fn name(&self) -> &'static str {
                self.name
            }

            pub fn data(&self) -> &'static [f32] {
                &OBJECT_DATA[self.start..self.end]
            }
        }

        static OBJECT_DATA: [f32; #num_points] = [ #(#points),* ];

        pub static OBJECTS: [Object; #num_objects] = [
            #(#objects_code),*
        ];
    };
    TokenStream::from(t)
}

#[proc_macro]
pub fn load_obj(tokens: TokenStream) -> TokenStream {
    match process_input(parse_macro_input!(tokens)) {
        Err(err) => TokenStream::from(err.to_compile_error()),
        Ok(o) => generate_output(o),
    }
}
