#![no_implicit_prelude]

use ::std::{
    assert_eq,
    convert::From,
    iter::Iterator,
    option::Option::Some,
    panic,
    result::Result::{Err, Ok},
    vec::Vec,
};

use ::obj::{IndexTuple, ObjData, SimplePolygon};
use ::proc_macro::TokenStream;
use ::quote::quote;
use ::syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, LitStr, Result, Token,
};

struct IdentPathMapping {
    ident: Ident,
    _eq: Token![=],
    path: LitStr,
}

impl Parse for IdentPathMapping {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(IdentPathMapping {
            ident: input.parse()?,
            _eq: input.parse()?,
            path: input.parse()?,
        })
    }
}

struct Input {
    mappings: Punctuated<IdentPathMapping, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Input {
            mappings: input.parse_terminated(IdentPathMapping::parse)?,
        })
    }
}

struct IdentObjMapping {
    ident: Ident,
    obj: ::obj::Obj,
}

fn process_input(input: Input) -> Result<Vec<IdentObjMapping>> {
    let cwd = ::std::env::current_dir().map_err(|err| {
        ::syn::Error::new(input.mappings.first().unwrap().ident.span(), err)
    })?;
    let mut objs = Vec::<IdentObjMapping>::new();
    for mapping in input.mappings {
        let p = mapping.path.value();
        let mut o: ::obj::Obj = ::obj::Obj::load_with_config(
            cwd.join(p),
            ::obj::LoadConfig { strict: true },
        )
        .map_err(|err| ::syn::Error::new(mapping.path.span(), err))?;
        o.load_mtls()
            .map_err(|err| ::syn::Error::new(mapping.path.span(), err))?;
        objs.push(IdentObjMapping {
            ident: mapping.ident,
            obj: o,
        });
    }
    Ok(objs)
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

fn generate_output(mappings: Vec<IdentObjMapping>) -> TokenStream {
    let mut points = Vec::with_capacity(2000);
    let mut objects_code = Vec::with_capacity(mappings.len());
    for mapping in mappings {
        let ident = mapping.ident;
        let data = &mapping.obj.data;
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
                pub static #ident: ObjectData = ObjectData {
                    name: #name,
                    buf: &VERTEX_DATA,
                    start: #obj_start,
                    end: #obj_end,
                };
            });
        }
    }
    let num_points = points.len();
    assert_eq!(num_points % 6, 0);

    let t = quote! {
        use crate::engine::scene::ObjectData;

        static VERTEX_DATA: [f32; #num_points] = [ #(#points),* ];

        #(#objects_code),*
    };
    TokenStream::from(t)
}

#[proc_macro]
pub fn load_objs(tokens: TokenStream) -> TokenStream {
    match process_input(parse_macro_input!(tokens)) {
        Err(err) => TokenStream::from(err.to_compile_error()),
        Ok(objs) => generate_output(objs),
    }
}
