use std::collections::HashSet;

use quote2::{proc_macro2::*, quote, Quote, Token};

macro_rules! parse {
    ($tokens: ident, $errors: ident, $msg: literal, $($tt:tt)*) => ({
        let msg = $msg;
        match $tokens.next().expect(msg) {
            $($tt)*,
            tt => {
                $errors.push(syn::Error::new(tt.span(), msg));
                continue;
            }
        }
    });
}

pub fn expand(input: TokenStream, output: &mut TokenStream) {
    let mut input = input.into_iter();
    let mut errors = Vec::new();
    let mut service_docs = String::new();

    while let Some(tt) = input.next() {
        match tt {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                let attrs = parse!(input, errors, "expected `[`", TokenTree::Group(v) if v.delimiter() == Delimiter::Bracket => v.stream().into_iter());
                parse_docs(&mut service_docs, output, attrs);
            }
            TokenTree::Ident(keyword) if keyword == "service" => {
                let service_name =
                    &parse!(input, errors, "expected service name", TokenTree::Ident(v) => v);

                quote!(output, {
                    struct #service_name;
                });

                let service_block =
                    parse!(input, errors, "expected { ... }", TokenTree::Group(v) => v);

                expand_service(
                    service_name,
                    service_block,
                    output,
                    &mut errors,
                    &service_docs,
                );
                service_docs.clear()
            }
            _ => output.extend(Some(tt)),
        }
    }
    if !errors.is_empty() {
        output.extend(errors.iter().map(|v| v.to_compile_error()));
    }
}

fn expand_service(
    service_name: &Ident,
    service_block: Group,
    output: &mut TokenStream,
    errors: &mut Vec<syn::Error>,
    service_docs: &str,
) {
    let mut tokens = service_block.stream().into_iter();
    let mut has_state = false;
    let mut funcs = Token(TokenStream::new());
    let mut items = Token(TokenStream::new());
    let mut func_types = TokenStream::new();
    let mut import_map = HashSet::new();
    let mut rpc_docs = String::new();

    while let Some(tt) = tokens.next() {
        match tt {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                let attrs = parse!(tokens, errors, "expected `[`", TokenTree::Group(v) if v.delimiter() == Delimiter::Bracket => v.stream().into_iter());
                parse_docs(&mut rpc_docs, &mut items, attrs);
            }
            TokenTree::Ident(keyword) if keyword == "rpc" => {
                let name = &parse!(tokens, errors, "expected rpc name", TokenTree::Ident(v) => v);

                parse!(tokens, errors, "expected `=`", TokenTree::Punct(v) if v.as_char() == '=' => v);
                let id = &parse!(tokens, errors, "expected rpc id", v @ (TokenTree::Literal(_) | TokenTree::Group(_)) => v);
                parse!(tokens, errors, "expected `;`", TokenTree::Punct(v) if v.as_char() == ';' => v);

                if let Some(prev) = import_map.replace(name.clone()) {
                    errors.push(syn::Error::new(
                        prev.span(),
                        format!("duplicate rpc: `{prev}`"),
                    ));
                    continue;
                }
                let rpc_ident = name.to_string();
                quote!(funcs, {
                    #id => Output::_produce(#name, state, cursor, transport),
                });
                let docs_str = rpc_docs.as_str();
                quote!(func_types, {
                    ::frpc::__private::fn_sig(&#name, &mut __costom_types, #id,  #rpc_ident, #docs_str),
                });
                rpc_docs.clear();
            }
            TokenTree::Ident(keyword) if keyword == "type" => {
                items.add(keyword);
                let next = tokens.next();
                if matches!(next, Some(TokenTree::Ident(ref ident)) if ident == "State") {
                    has_state = true;
                }
                items.extend(next);
            }
            _ => items.extend(Some(tt)),
        }
    }
    let service_ident = service_name.to_string();
    let func_types = Group::new(Delimiter::Bracket, func_types);

    let mut state = Token(TokenStream::new());
    if has_state {
        quote!(state, { State });
    } else {
        quote!(state, { () });
    };

    quote!(output, {
        const _: () = {
            #items

            impl #service_name {
                pub fn execute<'fut, TR: ::frpc::Transport + ::std::marker::Send>(
                    state: #state,
                    id: u16,
                    cursor: &'fut mut &[u8],
                    transport: &'fut mut TR,
                ) -> ::std::option::Option<::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()> + ::std::marker::Send + 'fut>>>
                {
                    use ::frpc::Output;
                    match id {
                        #funcs
                        _ => ::std::option::Option::None
                    }
                }
            }
        };

        #[cfg(debug_assertions)]
        impl ::std::convert::From<#service_name> for ::frpc::__private::frpc_message::TypeDef {
            fn from(_: #service_name) -> Self {
                let mut __costom_types = ::frpc::__private::frpc_message::CostomTypes::default();
                let funcs = ::std::vec::Vec::from(#func_types);
                Self::new(#service_ident, __costom_types, funcs, #service_docs)
            }
        }
    });
}

fn parse_docs(docs: &mut String, output: &mut TokenStream, mut attrs: token_stream::IntoIter) {
    match (attrs.next(), attrs.next(), attrs.next()) {
        (
            Some(TokenTree::Ident(name)),
            Some(TokenTree::Punct(p)),
            Some(TokenTree::Literal(doc)),
        ) if name == "doc" && p.as_char() == '=' => {
            docs.push_str(doc.to_string().trim_matches('"'));
            docs.push('\n');
        }
        (name, tt, tt2) => {
            let rest = quote(|o| o.extend(attrs));
            quote!(output, {
                #[#name #tt #tt2 #rest]
            });
        }
    }
}
