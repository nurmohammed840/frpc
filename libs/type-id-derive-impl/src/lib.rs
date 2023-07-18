use quote2::{
    proc_macro2::{Delimiter, Group},
    quote, IntoTokens, Quote, Token,
};
use syn::{
    __private::{Span, TokenStream2 as TokenStream},
    spanned::Spanned,
    *,
};

pub fn expand(
    crate_path: impl IntoTokens,
    input: &DeriveInput,
    output: &mut TokenStream,
    is_unit_enum: bool,
    enum_repr: Option<&String>,
) {
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
            let variants = data
                .variants
                .iter()
                .map(|v| (get_comments_from(&v.attrs), v.ident.to_string(), v));

            let mut discriminator = Discriminator::new();

            if is_unit_enum {
                let repr = match enum_repr {
                    Some(repr) => repr,
                    None => "isize",
                };
                for (doc, name, v) in variants {
                    let index = quote(|o| {
                        let index = discriminator.get(&v.discriminant);
                        let repr = Ident::new(repr, Span::call_site());
                        quote!(o, {
                            __crate::EnumRepr::#repr(#index),
                        });
                    });
                    quote!(body, {
                        __crate::UnitField::new(#doc, #name, #index),
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
                    let index = quote(|o| {
                        match enum_repr {
                            Some(repr) => {
                                let index = discriminator.get(&v.discriminant);
                                let repr = Ident::new(repr, Span::call_site());
                                quote!(o, {
                                    Some(__crate::EnumRepr::#repr(#index))
                                });
                            }
                            None => {
                                quote!(o, { None });
                            }
                        };
                    });
                    quote!(body, {
                        __crate::EnumField::new(#doc, #name, #index, __crate::EnumKind::#kind),
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

struct Discriminator {
    discriminant: Index,
    expr: Option<Expr>,
}

impl Discriminator {
    fn new() -> Self {
        Self {
            discriminant: Index::from(0),
            expr: None,
        }
    }

    fn get<'a>(
        &'a mut self,
        discriminant: &'a Option<(Token!(=), Expr)>,
    ) -> Token<impl FnOnce(&mut TokenStream) + 'a> {
        quote(move |o| match discriminant {
            Some((_, expr)) => {
                self.discriminant.index = 1;
                self.expr = Some(expr.clone());
                quote!(o, { #expr });
            }
            None => {
                let index = &self.discriminant;
                if let Some(expr) = &self.expr {
                    quote!(o, { #expr + #index });
                } else {
                    quote!(o, { #index });
                }
                self.discriminant.index += 1;
            }
        })
    }
}
