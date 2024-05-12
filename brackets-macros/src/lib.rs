#[macro_use]
extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_derive(ToJson)]
pub fn derive_to_json(item: TokenStream) -> TokenStream {
    let struct_ident = parse_macro_input!(item as ItemStruct);
    let struct_name = struct_ident.ident;
    let struct_fields = struct_ident.fields.iter().map(|x| {
        let x_ident = &x.ident;
        let x_key = x.ident.to_token_stream().to_string();
        quote! {
            output += "\"";
            output += #x_key;
            output += "\" : ";
            output += &self.#x_ident.to_json();
            output += ",";
        }
    });
    let generics = struct_ident.generics;
    let impl_types = generics.params.iter().map(|x| {
        let d = format_ident!("{}", x.to_token_stream().to_string().split(':').next().unwrap().trim());
        quote! {#d}
    }).collect::<Vec<_>>();
    let impl_insert = if impl_types.is_empty() { quote!{} } else {quote!{<#(#impl_types),*>}};

    let output_new = quote! {
        impl #generics brackets::ToJson for #struct_name #impl_insert {
            fn to_json(&self) -> String {
                let mut output = String::new();
                output += "{";
                #(#struct_fields)*
                output.pop();
                output += "}";
                return output;
            }
        }
    };

    output_new.into()
}

#[proc_macro_derive(FromJson)]
pub fn derive_from_json(item: TokenStream) -> TokenStream {
    let strct = parse_macro_input!(item as ItemStruct);
    let struct_name = &strct.ident;

    let fields_get = strct.fields.iter().map(|x| {
        let x_ident = &x.ident;
        let x_key = x.ident.to_token_stream().to_string();
        quote! {
            #x_ident: json.get(#x_key)?
        }
    });

    quote! {
        impl brackets::FromJson for #struct_name {
            fn from_json(json: &brackets::JsonObject) -> Result<#struct_name, brackets::JsonParseError> {
                Ok(#struct_name {
                    #(#fields_get),*
                })
            }
        }
    }.into()
}
