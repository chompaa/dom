use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, ItemFn, Path, Result,
};

enum ArgKind {
    Val(Ident),
    ValKind(Path, Ident),
}

struct Arg {
    kind: ArgKind,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind: Path = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let ident: syn::Ident = content.parse()?;
        if kind.is_ident("Val") {
            Ok(Arg {
                kind: ArgKind::Val(ident),
            })
        } else {
            Ok(Arg {
                kind: ArgKind::ValKind(kind, ident),
            })
        }
    }
}

pub(crate) fn expected_args_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Arg, syn::Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as ItemFn);

    let mut patterns = Vec::new();

    for arg in args {
        match arg.kind {
            ArgKind::Val(ident) => {
                patterns.push(quote! { #ident });
            }
            ArgKind::ValKind(kind, ident) => {
                patterns.push(quote! {
                    Val {
                        kind: ValKind::#kind(#ident),
                        ..
                    }
                });
            }
        }
    }

    let patterns_len = patterns.len();
    let patterns_combined = quote! {
        [#(#patterns),*] = &args[..#patterns_len]
    };

    let ItemFn {
        // The function signature
        sig,
        // The visibility specifier of this function
        vis,
        // The function block or body
        block,
        // Other attributes applied to this function
        attrs,
    } = input;

    // Extract statements in the body of the functions
    let statements = block.stmts;

    // Reconstruct the function as output using parsed input
    quote!(
        // Reapply all the other attributes on this function.
        // The compiler doesn't include the macro we are
        // currently working in this list.
        #(#attrs)*
        // Reconstruct the function declaration
        #vis #sig {
            let #patterns_combined else {
                return None;
            };

            // The rest of the function body
            #(#statements)*
        }
    )
    .into()
}
