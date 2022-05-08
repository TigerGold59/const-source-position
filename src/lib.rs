#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(proc_macro_span)]
extern crate proc_macro;

use proc_macro::{TokenStream, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Path};

/**
Takes a type path as an argument and provides the source code path as a type parameter to the last element of the path.
This has a very specific use case involving serde and large numbers of structs.
Example:
```ignore
    #![forbid(unsafe_code)]
    #![feature(adt_const_params)]
    #![allow(dead_code)]
    use std::fmt::{Display, Formatter};
    use std::fmt;

    extern crate const_source_position;

    // Phantom type
    enum Unconstructable<const SOURCE_LOCATION: &'static str> {}

    impl<const SOURCE_LOCATION: &'static str> Display for Unconstructable<SOURCE_LOCATION>  {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
        {
            write!(f, "{}", SOURCE_LOCATION)
        }
    }

    /*
        complicated type stuff, generics, macros, etc
    */

    fn generates_unconstructable_somehow() -> Box<dyn Display> {
        unimplemented!();
    }
```
*/
#[proc_macro]
pub fn source_line (item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(item as Path);

    let span = Span::call_site();
    let mut file_source =
    (*span
        .source_file()
        .path()
        .to_string_lossy()
    )
            .to_string();

    let line_column = span.source().start();

    file_source.push_str(&format!(":{}:{}", line_column.line, line_column.column));
    let lit = syn::LitStr::new(&file_source, span.into());

    let gen_arg = syn::GenericArgument::Const(syn::Expr::Lit(syn::ExprLit { attrs: vec![], lit: syn::Lit::Str(lit) }));

    let mut segments = path.segments.clone();

    let mut gen_args = syn::punctuated::Punctuated::<syn::GenericArgument, syn::token::Comma>::new();

    gen_args.push(gen_arg);

    segments.last_mut().unwrap().arguments = syn::PathArguments::AngleBracketed(
        syn::AngleBracketedGenericArguments {
            colon2_token: Some(
                syn::token::Colon2(path.span())
            ),
            lt_token: syn::token::Lt(path.span()),
            args: gen_args,
            gt_token: syn::token::Gt(path.span())
        }
    );

    let c = syn::Type::Path(syn::TypePath { qself: None, path: Path {
        leading_colon: path.leading_colon,
        segments,
    }});

    quote!(#c).into()
}