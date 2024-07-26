#![allow(unused)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn;

#[proc_macro_derive(Builder)]
pub fn derive(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let syn::DeriveInput {
        ident,
        data: syn::Data::Struct(syn::DataStruct { fields, .. }),
        ..
    } = input
    else {
        todo!()
    };
    let builder_ident = format_ident!("{}Builder", ident);

    let syn::Fields::Named(syn::FieldsNamed { named, .. }) = fields else {
        todo!()
    };
    let fields = named.iter();

    let struct_impl = quote! {
        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    ..Default::default()
                }
            }
        }
    };

    let option_fields = fields.clone().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: std::option::Option<#ty> }
    });
    let builder_def = quote! {
        #[derive(Default)]
        pub struct #builder_ident {
            #( #option_fields ),*
        }
    };

    let field_methods = fields.clone().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let optional_fields;
        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) = ty
        {
            optional_fields = segments.iter().filter_map(|seg| {
                let syn::PathSegment { ident, arguments } = seg;
                if ident == &format_ident!("Option") {
                    return None;
                }
                Some(seg)
            })
        }
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });
    let built_fields = fields.clone().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let err_msg = format!("{} missing", name.as_ref().unwrap());
        quote! {
            #name: self.#name.clone().ok_or(#err_msg)?
        }
    });

    let builder_impl = quote! {
        impl #builder_ident {
            #( #field_methods )*
            fn build(&self) -> Result<#ident, Box<dyn std::error::Error>> {
                Ok(#ident {
                    #( #built_fields ),*
                })
            }
        }
    };

    quote! {
            #struct_impl
            #builder_def
            #builder_impl
    }
    .into()
}
