mod expected_args;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn expected_args(args: TokenStream, item: TokenStream) -> TokenStream {
    expected_args::expected_args_impl(args, item)
}
