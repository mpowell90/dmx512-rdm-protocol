mod utils;

use crate::utils::{is_bool, is_u8};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields, Type, TypeArray};
use syn::{DeriveInput, parse_macro_input};

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else {
        return None;
    };

    let segment = tp.path.segments.last()?;
    if !segment.ident.eq("Option") {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    match args.args.first()? {
        syn::GenericArgument::Type(inner_ty) => Some(inner_ty),
        _ => None,
    }
}

fn handle_type_expr(ty: &Type, value_expr: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if is_u8(ty) {
        quote! {
            _buf[offset] = #value_expr;
            offset += 1;
        }
    } else if is_bool(ty) {
        quote! {
            _buf[offset] = (#value_expr) as u8;
            offset += 1;
        }
    } else {
        quote! {
            let bytes = (#value_expr).to_be_bytes();
            _buf[offset..offset + bytes.len()].copy_from_slice(&bytes);
            offset += bytes.len();
        }
    }
}

fn handle_type_array_expr(
    ta: &TypeArray,
    value_expr: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let len = &ta.len; // The 'N' in [u8; N]
    if is_u8(&ta.elem) {
        quote! {
            for i in 0..#len {
                _buf[offset + i] = (#value_expr)[i];
            }
            offset += #len;
        }
    } else if is_bool(&ta.elem) {
        quote! {
            for i in 0..#len {
                _buf[offset + i] = ((#value_expr)[i]) as u8;
            }
            offset += #len;
        }
    } else {
        let elem = &ta.elem;
        quote! {
            let elem_len = core::mem::size_of::<#elem>();
            for i in 0..#len {
                let bytes = (#value_expr)[i].to_be_bytes();
                let start = offset + (i * elem_len);
                _buf[start..start + elem_len].copy_from_slice(&bytes);
            }
            offset += (#len * elem_len);
        }
    }
}

fn encode_steps_for_field(f: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = f.ident.as_ref().expect("Named fields only");
    let field_expr = quote!(self.#field_name);

    if let Some(inner_ty) = option_inner_type(&f.ty) {
        let encode_inner = match inner_ty {
            Type::Array(ta) => handle_type_array_expr(ta, quote!(value)),
            _ => handle_type_expr(inner_ty, quote!(value)),
        };

        return quote! {
            if let Some(value) = #field_expr {
                #encode_inner
            }
        };
    }

    match &f.ty {
        // Specialized handling for [u8; N]
        Type::Array(ta) => handle_type_array_expr(ta, field_expr),
        Type::Path(tp) if tp.qself.is_none() => handle_type_expr(&tp.clone().into(), field_expr),
        // Standard primitives
        ty => handle_type_expr(ty, field_expr),
    }
}

fn decode_value_expr(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        // Specialized handling for [u8; N]
        Type::Array(ta) if is_u8(&ta.elem) => {
            let len = &ta.len; // The 'N' in [u8; N]
            quote! {
                {
                    let mut arr = [0u8; #len];
                    arr.copy_from_slice(&_bytes[offset..offset + #len]);
                    offset += #len;
                    arr
                }
            }
        }
        _ if is_u8(ty) => {
            quote! {
                {
                    let val = _bytes[offset];
                    offset += 1;
                    val
                }
            }
        }
        _ if is_bool(ty) => {
            quote! {
                {
                    let val = _bytes[offset] != 0;
                    offset += 1;
                    val
                }
            }
        }
        _ => {
            quote! {
                {
                    let len = core::mem::size_of::<#ty>();
                    let val = <#ty>::from_be_bytes(
                        _bytes[offset..offset + len]
                            .try_into()
                            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?
                    );
                    offset += len;
                    val
                }
            }
        }
    }
}

fn decode_steps_for_field(f: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = f.ident.as_ref().expect("Named fields only");
    let ty = &f.ty;

    if let Some(inner_ty) = option_inner_type(ty) {
        let decode_inner = decode_value_expr(inner_ty);
        return quote! {
            #field_name: {
                let remaining = _bytes.len().saturating_sub(offset);
                let needed = core::mem::size_of::<#inner_ty>();
                if remaining < needed {
                    None
                } else {
                    Some(#decode_inner)
                }
            }
        };
    }

    match ty {
        // Specialized handling for [u8; N]
        Type::Array(ta) if is_u8(&ta.elem) => {
            let len = &ta.len; // The 'N' in [u8; N]
            quote! {
                #field_name: {
                    let mut arr = [0u8; #len];
                    arr.copy_from_slice(&_bytes[offset..offset + #len]);
                    offset += #len;
                    arr
                }
            }
        }
        _ if is_u8(ty) => {
            quote! {
                #field_name: {
                    let val = _bytes[offset];
                    offset += 1;
                    val
                }
            }
        }
        _ if is_bool(ty) => {
            quote! {
                #field_name: {
                    let val = _bytes[offset] != 0;
                    offset += 1;
                    val
                }
            }
        }
        _ => {
            quote! {
                #field_name: {
                    let len = core::mem::size_of::<#ty>();
                    let val = <#ty>::from_be_bytes(
                        _bytes[offset..offset + len]
                            .try_into()
                            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?
                    );
                    offset += len;
                    val
                }
            }
        }
    }
}

#[proc_macro_derive(RdmGetRequestParameter)]
pub fn derive_rdm_get_request_parameter(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("RdmGetRequestParameterCodec only supports named fields"),
        },
        _ => panic!("Only structs supported"),
    };

    let encode_steps = fields.iter().map(encode_steps_for_field);

    let decode_steps = fields.iter().map(decode_steps_for_field);

    // let field_sizes = fields.iter().map(|f| {
    //     let ty = &f.ty;
    //     quote!(core::mem::size_of::<#ty>())
    // });

    let expanded = quote! {
        impl rdm_parameter_traits::RdmGetRequestParameterCodec for #name {
            fn get_request_encode_data(&self, _buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                // let required_size = 0 #(+ #field_sizes)*;
                // if _buf.len() < required_size { return Err(()); }

                let mut offset = 0;
                #(#encode_steps)*
                Ok(offset)
            }

            fn get_request_decode_data(_bytes: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
                // let required_size = 0 #(+ #field_sizes)*;
                // if _bytes.len() < required_size { return Err(()); }

                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(RdmGetResponseParameter)]
pub fn derive_rdm_get_response_parameter(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("RdmGetResponseParameterCodec only supports named fields"),
        },
        _ => panic!("Only structs supported"),
    };

    let encode_steps = fields.iter().map(encode_steps_for_field);

    let decode_steps = fields.iter().map(decode_steps_for_field);

    // let field_sizes = fields.iter().map(|f| {
    //     let ty = &f.ty;
    //     quote!(core::mem::size_of::<#ty>())
    // });

    let expanded = quote! {
        impl rdm_parameter_traits::RdmGetResponseParameterCodec for #name {
            fn get_response_encode_data(&self, _buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                // let required_size = 0 #(+ #field_sizes)*;
                // if _buf.len() < required_size { return Err(()); }

                let mut offset = 0;
                #(#encode_steps)*
                Ok(offset)
            }

            fn get_response_decode_data(_bytes: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
                // let required_size = 0 #(+ #field_sizes)*;
                // if _bytes.len() < required_size { return Err(()); }

                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(RdmSetRequestParameter)]
pub fn derive_rdm_set_request_parameter(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("RdmSetRequestParameterCodec only supports named fields"),
        },
        _ => panic!("Only structs supported"),
    };

    let encode_steps = fields.iter().map(encode_steps_for_field);

    let decode_steps = fields.iter().map(decode_steps_for_field);

    // let field_sizes = fields.iter().map(|f| {
    //     let ty = &f.ty;
    //     quote!(core::mem::size_of::<#ty>())
    // });

    let expanded = quote! {
        impl rdm_parameter_traits::RdmSetRequestParameterCodec for #name {
            fn set_request_encode_data(&self, _buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                // let required_size = 0 #(+ #field_sizes)*;
                // if _buf.len() < required_size { return Err(()); }

                let mut offset = 0;
                #(#encode_steps)*
                Ok(offset)
            }

            fn set_request_decode_data(_bytes: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
                // let required_size = 0 #(+ #field_sizes)*;
                // if _bytes.len() < required_size { return Err(()); }

                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(RdmSetResponseParameter)]
pub fn derive_rdm_set_response_parameter(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("RdmSetResponseParameterCodec only supports named fields"),
        },
        _ => panic!("Only structs supported"),
    };

    let encode_steps = fields.iter().map(encode_steps_for_field);

    let decode_steps = fields.iter().map(decode_steps_for_field);

    // let field_sizes = fields.iter().map(|f| {
    //     let ty = &f.ty;
    //     quote!(core::mem::size_of::<#ty>())
    // });

    let expanded = quote! {
        impl rdm_parameter_traits::RdmSetResponseParameterCodec for #name {
            fn set_response_encode_data(&self, _buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                // let required_size = 0 #(+ #field_sizes)*;
                // if _buf.len() < required_size { return Err(()); }

                let mut offset = 0;
                #(#encode_steps)*
                Ok(offset)
            }

            fn set_response_decode_data(_bytes: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
                // let required_size = 0 #(+ #field_sizes)*;
                // if _bytes.len() < required_size { return Err(()); }

                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
