use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

#[proc_macro_derive(Getters)]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("Getters only work on structs"),
    };

    let mut getters = Vec::new();

    if let Fields::Named(named_fields) = fields {
        for f in &named_fields.named {
            let field_name = f.ident.as_ref().unwrap();
            let field_type = &f.ty;

            let is_primitive = match field_type {
                Type::Path(type_path) => {
                    if type_path.qself.is_none()
                        && type_path.path.leading_colon.is_none()
                        && type_path.path.segments.len() == 1
                    {
                        let segment_ident = &type_path.path.segments[0].ident;
                        matches!(
                            segment_ident.to_string().as_str(),
                            "u8" | "u16"
                                | "u32"
                                | "u64"
                                | "u128"
                                | "usize"
                                | "i8"
                                | "i16"
                                | "i32"
                                | "i64"
                                | "i128"
                                | "isize"
                                | "f32"
                                | "f64"
                                | "bool"
                                | "char"
                        )
                    } else {
                        false
                    }
                }
                _ => false,
            };

            getters.push(if is_primitive {
                quote! {
                    pub fn #field_name(&self) -> #field_type {
                        self.#field_name
                    }
                }
            } else {
                quote! {
                    pub fn #field_name(&self) -> &#field_type {
                        &self.#field_name
                    }
                }
            });
        }
    }

    let expanded = quote! {
        impl #name {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}
