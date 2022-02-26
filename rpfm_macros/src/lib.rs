//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Small crate to put the procedural macros used in RPFM.
!*/

#![crate_type = "proc-macro"]
use proc_macro::TokenStream;
use proc_macro2::Span;

use quote::quote;
use syn::{Data, DeriveInput, Ident, parse_macro_input};

/// Macro to generate automatic clone getters for a Struct.
#[proc_macro_derive(GetClone)]
pub fn getter_clone(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let struct_name = &ast.ident;
    match ast.data {
        Data::Struct(s) => {

            let field_types : Vec<_> = s.fields
                .iter()
                .map(|x| x.ty.clone()).collect();

            let field_names : Vec<_> = s.fields
                .iter()
                .map(|x| x.ident.clone().unwrap()).collect();

            let function_names = field_names
                .iter()
                .map(|x| Ident::new(format!("get_{}", x).as_str(), Span::call_site()));

            let quoted_code = quote!{

                #[allow(dead_code)]
                impl #struct_name {
                    #(
                        pub fn #function_names(&self) -> #field_types {
                            self.#field_names.clone()
                        }
                    )*
                }
            };
            TokenStream::from(quoted_code)
        }

        // not a struct
        _ => "".parse().unwrap()
    }
}

/// Macro to generate automatic reference getters for a Struct.
#[proc_macro_derive(GetRef)]
pub fn getter_ref(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let struct_name = &ast.ident;
    match ast.data {
        Data::Struct(s) => {

            let field_types : Vec<_> = s.fields
                .iter()
                .map(|x| x.ty.clone()).collect();

            let field_names : Vec<_> = s.fields
                .iter()
                .map(|x| x.ident.clone().unwrap()).collect();

            let function_names = field_names
                .iter()
                .map(|x| Ident::new(format!("get_ref_{}", x).as_str(), Span::call_site()));

            let quoted_code = quote!{

                #[allow(dead_code)]
                impl #struct_name {
                    #(
                        pub fn #function_names(&self) -> &#field_types {
                            &self.#field_names
                        }
                    )*
                }
            };
            TokenStream::from(quoted_code)
        }

        // not a struct
        _ => "".parse().unwrap()
    }
}

/// Macro to generate automatic mutable reference getters for a Struct.
#[proc_macro_derive(GetRefMut)]
pub fn getter_ref_mut(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let struct_name = &ast.ident;
    match ast.data {
        Data::Struct(s) => {

            let field_types : Vec<_> = s.fields
                .iter()
                .map(|x| x.ty.clone()).collect();

            let field_names : Vec<_> = s.fields
                .iter()
                .map(|x| x.ident.clone().unwrap()).collect();

            let function_names = field_names
                .iter()
                .map(|x| Ident::new(format!("get_ref_mut_{}", x).as_str(), Span::call_site()));

            let quoted_code = quote!{

                #[allow(dead_code)]
                impl #struct_name {
                    #(
                        pub fn #function_names(&mut self) -> &mut #field_types {
                            &mut self.#field_names
                        }
                    )*
                }
            };
            TokenStream::from(quoted_code)
        }

        // not a struct
        _ => "".parse().unwrap()
    }
}

/// Macro to generate automatic setters for a Struct.
#[proc_macro_derive(Set)]
pub fn setter(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let struct_name = &ast.ident;
    match ast.data {
        Data::Struct(s) => {

            let field_types : Vec<_> = s.fields
                .iter()
                .map(|x| x.ty.clone()).collect();

            let field_names : Vec<_> = s.fields
                .iter()
                .map(|x| x.ident.clone().unwrap()).collect();

            let function_names = field_names
                .iter()
                .map(|x| Ident::new(format!("set_{}", x).as_str(), Span::call_site()));

            let quoted_code = quote!{

                #[allow(dead_code)]
                impl #struct_name {
                    #(
                        pub fn #function_names(&mut self, #field_names: #field_types) {
                            self.#field_names = #field_names;
                        }
                    )*
                }
            };
            TokenStream::from(quoted_code)
        }

        // not a struct
        _ => "".parse().unwrap()
    }
}
