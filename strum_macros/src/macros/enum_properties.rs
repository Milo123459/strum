use std::{fs::File, io::Write};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};
use std::fmt::Debug;

use crate::helpers::{non_enum_error, HasStrumVariantProperties, HasTypeProperties};

pub fn enum_properties_inner(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => return Err(non_enum_error()),
    };
    let type_properties = ast.get_type_properties()?;
    let strum_module_path = type_properties.crate_module_path();

    let mut arms = Vec::new();
    for variant in variants {
        let ident = &variant.ident;
        let variant_properties = variant.get_variant_properties()?;
        let mut string_arms = Vec::new();
        let mut bool_arms = Vec::new();
        let mut int_arms = Vec::new();
        // But you can disable the messages.
        if variant_properties.disabled.is_some() {
            continue;
        }


        let mut file = File::create(format!("variant-{}.txt", variant.ident.to_string())).unwrap();
        file.write_all(format!("{:#?}", variant.clone()).as_bytes());

        let params = match variant.fields {
            Fields::Unit => quote! {},
            Fields::Unnamed(..) => quote! { (..) },
            Fields::Named(..) => quote! { {..} },
        };

        for (key, value) in variant_properties.string_props {
            string_arms.push(quote! { #key => ::core::option::Option::Some( #value )});
        }

        for (key, value) in variant_properties.bool_props {
            bool_arms.push(quote! { #key => ::core::option::Option::Some( #value )});
        }

        for (key, value) in variant_properties.int_props {
            int_arms.push(quote! { #key => ::core::option::Option::Some( #value )});
        }


        string_arms.push(quote! { _ => ::core::option::Option::None },);

        arms.push(quote! {
            &#name::#ident #params => {
                match prop {
                    #(#string_arms),*
                    #(#bool_arms),*
                    #(#int_arms),*
                }
            }
        });

        
    }

    if arms.len() < variants.len() {
        arms.push(quote! { _ => ::core::option::Option::None });
    }
    let q = &arms.to_vec().iter().map(|a| a.to_string()).collect::<Vec<_>>().join("\n");
    let mut file = File::create("q.txt").unwrap();
    file.write_all(q.as_bytes());
    Ok(quote! {
        impl #impl_generics #strum_module_path::EnumProperty for #name #ty_generics #where_clause {
            fn get_str(&self, prop: &str) -> ::core::option::Option<&'static str> {
                match self {
                    #(#arms),*
                }
            }
        }
    })
}
