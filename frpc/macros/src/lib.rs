mod declare;

use databuf_derive_impl::Expand;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote2::{proc_macro2, quote, Quote};
use syn::{parse_macro_input, DeriveInput};

macro_rules! crate_path {
    [$($tt:tt)*] => ({
        let mut path = TokenStream2::new();
        quote!(path, { $($tt)* });
        path
    });
}

fn message_expand(input: TokenStream, f: impl FnOnce(Expand)) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let mut output = TokenStream2::new();

    type_id_derive_impl::expand(
        crate_path!(::frpc::__private::frpc_message::type_id),
        &input,
        &mut output,
    );
    f(Expand::new(
        crate_path!(::frpc::databuf),
        &input,
        &mut output,
    ));
    output.into()
}

/// Represent both [Input] + [Output]
#[proc_macro_derive(Message)]
pub fn message(input: TokenStream) -> TokenStream {
    message_expand(input, |mut expand| {
        expand.encoder();
        expand.decoder();
    })
}

#[proc_macro_derive(Input)]
pub fn input(input: TokenStream) -> TokenStream {
    message_expand(input, |mut expand| {
        expand.decoder();
    })
}

#[proc_macro_derive(Output)]
pub fn output(input: TokenStream) -> TokenStream {
    message_expand(input, |mut expand| {
        expand.encoder();
    })
}

#[proc_macro]
pub fn declare(input: TokenStream) -> TokenStream {
    let mut output = TokenStream2::new();
    declare::expand(TokenStream2::from(input), &mut output);
    output.into()
}
