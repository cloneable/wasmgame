#![no_std]

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn test_macro(_item: TokenStream) -> TokenStream {
    "pub static TEST_DATA: [f32; 1] = [132.0];".parse().unwrap()
}
