#![forbid(unsafe_code)]
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use quote::quote;
use std::{default::Default, num::ParseIntError};
use syn::{
    parse_str, punctuated::Punctuated, token::Comma, Attribute, DeriveInput, Expr, ExprLit, Field,
    GenericParam, Ident, Lit, Type, TypePath,
};

const ATTR_SKIP: &str = "skip";
const ATTR_WHEN: &str = "when";
const ATTR_LEN: &str = "len";

//TODO: union

#[proc_macro_derive(Snd, attributes(skip, when, len))]
pub fn send(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(item).unwrap();
    match ast.data {
        syn::Data::Enum(_) => send_enum(&ast),
        syn::Data::Union(_) => TokenStream::new(),
        syn::Data::Struct(_) => send_struct(&ast),
    }
}

#[proc_macro_derive(Rcv, attributes(skip, when, len))]
pub fn receive(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(item).unwrap();
    // eprintln!("{:#?}", &ast);

    match ast.data {
        syn::Data::Enum(_) => receive_enum(&ast),
        syn::Data::Union(_) => TokenStream::new(),
        syn::Data::Struct(_) => receive_struct(&ast),
    }
}

#[inline]
fn send_struct(ast: &syn::DeriveInput) -> TokenStream {
    let id_name = &ast.ident;
    let fields = get_named_fields(ast);

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let transient = get_attr(&f.attrs, ATTR_SKIP);

        let attr_len = get_attr(&f.attrs, ATTR_LEN);

        let attr = get_attr(&f.attrs, ATTR_WHEN);
        if transient.is_none() {
            let ty = &f.ty;
            match ty {
                Type::Reference(_) => quote! {
                    writer.snd(self.#name)?
                },
                Type::Path(ref p) if is_option(p) && attr.is_some() => {
                    quote! {
                        if let Some(ref v) = self.#name {
                            //we don't check the "when" attribute here since it needs to use self.xx
                            writer.snd(v)?
                        }
                    }
                }
                _ => {
                    if let Some(v) = attr_len {
                        let len_q = match get_attr_len(v) {
                            AttrLen::Ident(id) => quote! {
                                self.#id
                            },
                            AttrLen::LitInt(i) => quote! {
                                #i
                            },
                            AttrLen::None => panic!("Invalid len attribute"),
                        };
                        quote! {
                            resend::IntoWriter::into_writer(&self.#name, writer, #len_q as usize)?;
                        }
                    } else {
                        quote! {
                            writer.snd(&(self.#name))?
                        }
                    }
                }
            }
        } else {
            Default::default() //empty TokenStream
        }
    });

    let generics = get_lifetime(ast);

    let life = quote! {
        <#(#generics,)*>
    };

    let gen = quote! {
        impl#life resend::Sendable for &#id_name#life {
            #[inline]
            fn snd_to<S: resend::Sender>(&self, writer: &mut S) -> resend::Result<()> {
                #(#build_fields;)*
                Ok(())
            }
        }

    };
    gen.into()
}
#[inline]
fn send_enum(ast: &syn::DeriveInput) -> TokenStream {
    if let syn::Data::Enum(data) = &ast.data {
        let id = &ast.ident;
        let mut tag_value = 0;

        let mut tag_type = Ident::new("u8", Span::call_site());
        if let Some(attr) = get_attr(&ast.attrs, "repr") {
            if let Some(meta) = get_attr_meta(attr) {
                tag_type = meta;
            }
        };

        let arms = data.variants.iter().map(|va| {
            let id_item = &va.ident;
            if let Some((
                _,
                Expr::Lit(ExprLit {
                    lit: Lit::Int(v), ..
                }),
            )) = &va.discriminant
            {
                tag_value = parse_int(&v.to_string()).unwrap();
            }

            let path = if let syn::Fields::Unnamed(f) = &va.fields {
                if let Some(ff) = f.unnamed.first() {
                    if let syn::Type::Path(p) = &ff.ty {
                        Some(p)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let tmp_value = if tag_type == "u8" {
                Literal::u8_suffixed(tag_value as u8)
            } else if tag_type == "u16" {
                Literal::u16_suffixed(tag_value as u16)
            } else {
                Literal::u32_suffixed(tag_value)
            };

            tag_value += 1;

            if path.is_some() {
                quote! {
                    #id::#id_item(v) => {
                        writer.snd(#tmp_value)?;
                        writer.snd(v)?;
                    }
                }
            } else {
                quote! {
                    #id::#id_item => writer.snd(#tmp_value)?
                }
            }
        });

        let gen = quote! {
            impl resend::Sendable for &#id {
                #[inline]
                fn snd_to<S: resend::Sender>(&self, writer: &mut S) -> resend::Result<()> {
                    match self {
                        #(#arms,)*
                    }
                    Ok(())
                }
            }
        };

        gen.into()
    } else {
        panic!("Enum expected");
    }
}

//region receive
#[inline]
fn receive_struct(ast: &syn::DeriveInput) -> TokenStream {
    let id_name = &ast.ident;
    let fields = get_named_fields(ast);

    let mut names = Vec::with_capacity(fields.len());
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        names.push(name);
        let transient = get_attr(&f.attrs, ATTR_SKIP);
        if transient.is_some() {
            quote! {
                let #name = std::default::Default::default()
            }
        } else if let Some(attr) = get_attr(&f.attrs, ATTR_WHEN) {
            let exp = get_when_args(attr);
            quote! {
                let #name = if #exp {
                    Some(reader.rcv()?)
                }else{
                    None
                }
            }
        } else if let Some(v) = get_attr(&f.attrs, ATTR_LEN) {
            let len_q = match get_attr_len(v) {
                AttrLen::Ident(id) => quote! {
                    #id
                },
                AttrLen::LitInt(i) => quote! {
                    #i
                },
                AttrLen::None => panic!("Invalid len attribute"),
            };
            quote! {
                let #name = resend::FromReader::from_reader(reader, #len_q as usize)?;
            }
        } else {
            quote! {
                let #name = reader.rcv()?
            }
        }
    });

    let generics = get_lifetime(ast);

    let life = quote! {
        <#(#generics,)*>
    };

    let gen = quote! {
        impl#life resend::Receivable for #id_name#life {
            #[inline]
            fn rcv_from<R: resend::Receiver>(reader: &mut R) -> resend::Result<Self> {
                #(#build_fields;)*

                Ok(
                    #id_name{
                        #(#names,)*
                    }
                )
            }
        }

    };
    gen.into()
}

#[inline]
fn receive_enum(ast: &syn::DeriveInput) -> TokenStream {
    if let syn::Data::Enum(data) = &ast.data {
        let id = &ast.ident;
        let mut tag_value = 0;

        let mut tag_type = Ident::new("u8", Span::call_site());
        if let Some(attr) = get_attr(&ast.attrs, "repr") {
            if let Some(meta) = get_attr_meta(attr) {
                tag_type = meta;
            }
        };

        let arms = data.variants.iter().map(|va| {
            let id_item = &va.ident;
            if let Some((
                _,
                Expr::Lit(ExprLit {
                    lit: Lit::Int(v), ..
                }),
            )) = &va.discriminant
            {
                tag_value = parse_int(&v.to_string()).unwrap();
            }

            let path = if let syn::Fields::Unnamed(f) = &va.fields {
                if let Some(ff) = f.unnamed.first() {
                    if let syn::Type::Path(p) = &ff.ty {
                        Some(p)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            let tmp_value = if tag_type == "u8" {
                Literal::u8_unsuffixed(tag_value as u8)
            } else if tag_type == "u16" {
                Literal::u16_unsuffixed(tag_value as u16)
            } else {
                Literal::u32_unsuffixed(tag_value)
            };

            tag_value += 1;

            if let Some(p) = path {
                quote! {
                    #tmp_value => {
                        let t: #p = reader.rcv()?;
                        Ok(#id::#id_item(t))
                    }
                }
            } else {
                quote! {
                    #tmp_value => Ok(#id::#id_item)
                }
            }
        });

        let gen = quote! {
            impl resend::Receivable for #id {
                #[inline]
                fn rcv_from<R: resend::Receiver>(reader: &mut R) -> resend::Result<Self> {
                    let tag: #tag_type = reader.rcv()?;
                    match tag {
                        #(#arms,)*
                        _ => Err(resend::error::Error::InvalidTag(tag as u32)),
                    }
                }
            }
        };

        gen.into()
    } else {
        panic!("Enum expected");
    }
}

//endregion

#[inline]
fn get_named_fields(ast: &syn::DeriveInput) -> &Punctuated<Field, Comma> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        panic!("Struct only")
    }
}
#[inline]
fn get_lifetime(ast: &syn::DeriveInput) -> Vec<&GenericParam> {
    let mut generics = Vec::new();
    for g in &ast.generics.params {
        generics.push(g);
    }

    generics
}

#[inline]
fn get_attr<'a>(attrs: &'a [Attribute], value: &str) -> Option<&'a Attribute> {
    attrs.iter().find(|&attr| attr.path.is_ident(value))
}

enum AttrLen {
    Ident(syn::Ident),
    LitInt(syn::LitInt),
    None,
}
#[inline]
fn get_attr_len(attr: &Attribute) -> AttrLen {
    if attr.path.is_ident(ATTR_LEN) {
        if let Ok(syn::Meta::List(l)) = attr.parse_meta() {
            if let Some(v) = l.nested.first() {
                match v {
                    syn::NestedMeta::Meta(m) => {
                        return AttrLen::Ident(m.path().get_ident().unwrap().clone());
                    }
                    syn::NestedMeta::Lit(syn::Lit::Int(i)) => {
                        return AttrLen::LitInt(i.clone());
                    }
                    _ => (),
                }
            }
        }
    }
    AttrLen::None
}
#[inline]
fn get_attr_meta(attr: &Attribute) -> Option<Ident> {
    if let Ok(syn::Meta::List(l)) = attr.parse_meta() {
        if let Some(syn::NestedMeta::Meta(m)) = l.nested.first() {
            if let Some(v) = m.path().get_ident() {
                return Some(v.clone());
            }
        }
    }
    None
}

#[inline]
fn get_when_args(attr: &Attribute) -> Expr {
    // attr.parse_meta return error here, so we use to_string
    let mut s = attr.tokens.to_string();
    if s.len() > 2 {
        //must be (x) at least
        let first = s.remove(0);
        let last = s.pop().unwrap();
        if first == '(' && last == ')' {
            return parse_str::<Expr>(&s).unwrap();
        }
    }
    panic!("Inlvalid args: {}", s);
}

#[inline]
fn is_option(p: &TypePath) -> bool {
    let segs = &p.path.segments;
    match segs.len() {
        1 => segs[0].ident == "Option",
        2 => segs[0].ident == "option" && segs[1].ident == "Option",
        3 => segs[0].ident == "std" && segs[1].ident == "option" && segs[2].ident == "Option",
        _ => false,
    }
}
#[inline]
fn parse_int(s: &str) -> Result<u32, ParseIntError> {
    if let Some(stripped) = s.strip_prefix("0x") {
        u32::from_str_radix(stripped, 16)
    } else if let Some(stripped) = s.strip_prefix("0b") {
        u32::from_str_radix(stripped, 2)
    } else if let Some(stripped) = s.strip_prefix("0o") {
        u32::from_str_radix(stripped, 8)
    } else {
        s.parse::<u32>()
    }
}
