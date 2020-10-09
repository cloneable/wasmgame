#![no_implicit_prelude]

use ::std::convert::From;
use ::std::fs::File;
use ::std::io::Read;
use ::std::result::{Result::Err, Result::Ok};
use ::std::string::String;

use ::proc_macro::TokenStream;
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

fn process_input(input: Input) -> Result<()> {
    let path = input.obj_path.value();
    let mut data = String::new();
    {
        let cwd = ::std::env::current_dir()
            .map_err(|err| ::syn::Error::new(input.obj_path.span(), err))?;
        let mut f = File::open(cwd.join(path))
            .map_err(|err| ::syn::Error::new(input.obj_path.span(), err))?;
        f.read_to_string(&mut data)
            .map_err(|err| ::syn::Error::new(input.obj_path.span(), err))?;
    }
    let buf = ::std::io::BufReader::new(data)
    let o: ::obj::Obj = ::obj::load_obj(buf)
        .map_err(|err| ::syn::Error::new(input.obj_path.span(), err))?;

    Ok(())
}

#[proc_macro]
pub fn load_obj(tokens: TokenStream) -> TokenStream {
    match process_input(parse_macro_input!(tokens)) {
        Err(err) => TokenStream::from(err.to_compile_error()),
        Ok(_) => "\"OBJ\"".parse().unwrap(),
    }
}
