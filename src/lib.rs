mod parse;
mod summon;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn featruct(input: TokenStream, item: TokenStream) -> TokenStream {
    todo!()
}
