use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

fn encode_step(f: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &f.ident;
    let ty = &f.ty;
    quote! {
        offset += <#ty as RdmParameterData>::encode_rdm_parameter_data(&self.#field_name, &mut buf[offset..])?;
    }
}

fn decode_step(f: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &f.ident;
    let ty = &f.ty;
    quote! {
        #field_name: {
            let val = <#ty as RdmParameterData>::decode_rdm_parameter_data(&buf[offset..])?;
            offset += <#ty as RdmParameterData>::size_of(&val);
            val
        }
    }
}

fn field_size(f: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &f.ident;
    let ty = &f.ty;
    quote!(<#ty as RdmParameterData>::size_of(&self.#field_name))
}

#[proc_macro_derive(RdmDiscoveryRequestParameter)]
pub fn derive_rdm_discovery_request_parameter(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("RdmDiscoveryRequestParameterCodec only supports named fields"),
        },
        _ => panic!("Only structs supported"),
    };

    let encode_steps = fields.iter().map(encode_step);

    let decode_steps = fields.iter().map(decode_step);

    let field_sizes = fields.iter().map(field_size);

    let expanded = quote! {
        impl rdm_parameter_traits::RdmDiscoveryRequestParameterCodec for #name {
            fn size_of(&self) -> usize {
                #(#field_sizes)*
            }
            fn discovery_request_encode_data(&self, buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_parameter_traits::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                let mut offset = 0;

                #(#encode_steps)*

                Ok(offset)
            }

            fn discovery_request_decode_data(buf: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(RdmDiscoveryResponseParameter)]
pub fn derive_rdm_discovery_response_parameter(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("RdmDiscoveryResponseParameterCodec only supports named fields"),
        },
        _ => panic!("Only structs supported"),
    };

    let encode_steps = fields.iter().map(encode_step);

    let decode_steps = fields.iter().map(decode_step);

    let field_sizes = fields.iter().map(field_size);

    let expanded = quote! {
        impl rdm_parameter_traits::RdmDiscoveryResponseParameterCodec for #name {
            fn size_of(&self) -> usize {
                0 #( + #field_sizes)*
            }

            fn discovery_response_encode_data(&self, buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_parameter_traits::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                let mut offset = 0;

                #(#encode_steps)*

                Ok(offset)
            }

            fn discovery_response_decode_data(buf: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }
    };

    TokenStream::from(expanded)
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

    let encode_steps = fields.iter().map(encode_step);

    let decode_steps = fields.iter().map(decode_step);

    let field_sizes = fields.iter().map(field_size);

    let expanded = quote! {
        impl rdm_parameter_traits::RdmGetRequestParameterCodec for #name {
            fn size_of(&self) -> usize {
                0 #( + #field_sizes)*
            }

            fn get_request_encode_data(&self, buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_parameter_traits::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                let mut offset = 0;

                #(#encode_steps)*

                Ok(offset)
            }

            fn get_request_decode_data(buf: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
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

    let encode_steps = fields.iter().map(encode_step);

    let decode_steps = fields.iter().map(decode_step);

    let field_sizes = fields.iter().map(field_size);

    let expanded = quote! {
        impl rdm_parameter_traits::RdmGetResponseParameterCodec for #name {
            fn size_of(&self) -> usize {
                0 #( + #field_sizes)*
            }

            fn get_response_encode_data(&self, buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_parameter_traits::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                let mut offset = 0;

                #(#encode_steps)*

                Ok(offset)
            }

            fn get_response_decode_data(buf: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
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

    let encode_steps = fields.iter().map(encode_step);

    let decode_steps = fields.iter().map(decode_step);

    let field_sizes = fields.iter().map(field_size);

    let expanded = quote! {
        impl rdm_parameter_traits::RdmSetRequestParameterCodec for #name {
            fn size_of(&self) -> usize {
                0 #( + #field_sizes)*
            }

            fn set_request_encode_data(&self, buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_parameter_traits::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                let mut offset = 0;

                #(#encode_steps)*

                Ok(offset)
            }

            fn set_request_decode_data(buf: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
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

    let encode_steps = fields.iter().map(encode_step);

    let decode_steps = fields.iter().map(decode_step);

    let field_sizes = fields.iter().map(field_size);

    let expanded = quote! {
        impl rdm_parameter_traits::RdmSetResponseParameterCodec for #name {
            fn size_of(&self) -> usize {
                0 #( + #field_sizes)*
            }

            fn set_response_encode_data(&self, buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_parameter_traits::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                let mut offset = 0;

                #(#encode_steps)*

                Ok(offset)
            }

            fn set_response_decode_data(buf: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
