use proc_macro::TokenStream;
use syn::DeriveInput;
mod error_info;
use error_info::process_error_info;

#[proc_macro_derive(ToErrorInfo, attributes(error_info))]
pub fn derive_to_error_info(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    process_error_info(input).into()
}
