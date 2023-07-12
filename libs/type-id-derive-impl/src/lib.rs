use quote2::{
    proc_macro2::{Delimiter, Group},
    quote, IntoTokens, Quote, Token,
};
use syn::{
    __private::{Span, TokenStream2 as TokenStream},
    spanned::Spanned,
    *,
};

pub fn expand(crate_path: impl IntoTokens, input: &DeriveInput, output: &mut TokenStream) {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = input;

    let doc = get_comments_from(attrs);
    let fmt_str = format!("{{}}::{ident}");

    if let Some(param) = generics.type_params().next() {
        return output.extend(
            Error::new(param.span(), "Support for generic type isn't complete yet.")
                .to_compile_error(),
        );
    }

    let mut body = TokenStream::new();
    let kind = match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                to_object(&mut body, fields);
                "Struct"
            }
            Fields::Unnamed(fields) => {
                to_tuple(&mut body, fields);
                "Tuple"
            }
            Fields::Unit => panic!("`{ident}` struct needs at most one field"),
        },
        Data::Enum(data) => {
            let is_unit = data
                .variants
                .iter()
                .all(|v| v.discriminant.is_some() || matches!(v.fields, Fields::Unit));

            let variants = data
                .variants
                .iter()
                .map(|v| (get_comments_from(&v.attrs), v.ident.to_string(), v));

            if is_unit {
                let mut value: isize = -1;
                for (doc, name, v) in variants {
                    value = match &v.discriminant {
                        Some((_, expr)) => parse_int(expr),
                        None => value + 1,
                    };
                    quote!(body, {
                        __crate::UnitField::new(#doc, #name, #value),
                    });
                }
                "Unit"
            } else {
                for (doc, name, v) in variants {
                    let kind = quote(|o| match &v.fields {
                        Fields::Named(fields) => {
                            let body = quote(|o| to_object(o, fields));
                            quote!(o, { Struct(::std::vec![#body]) });
                        }
                        Fields::Unnamed(fields) => {
                            let body = quote(|o| to_tuple(o, fields));
                            quote!(o, { Tuple(::std::vec![#body]) });
                        }
                        Fields::Unit => {
                            quote!(o, { Unit });
                        }
                    });
                    quote!(body, {
                        __crate::EnumField::new(#doc, #name, __crate::EnumKind::#kind),
                    });
                }
                "Enum"
            }
        }
        Data::Union(_) => panic!("`Message` implementation for `union` is not yet stabilized"),
    };

    let kind = Ident::new(kind, Span::call_site());
    let body = Token(Group::new(Delimiter::Bracket, body));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote!(output, {
        const _: () = {
            use #crate_path as __crate;
            impl #impl_generics __crate::TypeId for #ident #ty_generics #where_clause {
                fn ty(__c: &mut __crate::CostomTypes) -> __crate::Ty {
                    __c.register(
                        ::std::format!(#fmt_str, ::std::module_path!()),
                        |__c| __crate::CustomTypeKind::#kind(__crate::CustomType::new(#doc, ::std::vec!#body))
                    )
                }
            }
        };
    });
}

fn to_tuple(body: &mut TokenStream, fields: &FieldsUnnamed) {
    for Field { attrs, ty, .. } in &fields.unnamed {
        let doc: String = get_comments_from(attrs);
        quote!(body, {
            __crate::TupleField::new(#doc, <#ty as __crate::TypeId>::ty(__c)),
        });
    }
}

fn to_object(body: &mut TokenStream, fields: &FieldsNamed) {
    for Field {
        attrs, ident, ty, ..
    } in &fields.named
    {
        let doc = get_comments_from(attrs);
        let ident = ident.as_ref().map(|v| v.to_string());
        quote!(body, {
            __crate::StructField::new(#doc, #ident, <#ty as __crate::TypeId>::ty(__c)),
        });
    }
}

fn parse_int(expr: &Expr) -> isize {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Int(int) => int.base10_parse().unwrap(),
            _ => panic!("Expect integer"),
        },
        _ => panic!("Not a number"),
    }
}

fn get_comments_from(attrs: &Vec<Attribute>) -> String {
    let mut string = String::new();
    for attr in attrs {
        if let Meta::NameValue(MetaNameValue { path, value, .. }) = &attr.meta {
            if path.is_ident("doc") {
                if let Expr::Lit(expr) = value {
                    if let Lit::Str(data) = &expr.lit {
                        string += &data.value();
                        string += "\n"
                    }
                }
            }
        }
    }
    string
}
