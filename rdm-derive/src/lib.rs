use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Fields, parse_macro_input};

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

#[proc_macro_attribute]
pub fn rdm_request_parameter(args: TokenStream, input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = &input.ident;

    let fields = match input.clone().data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("RdmRequestParameterCodec only supports named fields"),
        },
        _ => panic!("Only structs supported"),
    };

    let mut pid_expr: Option<Expr> = None;
    let mut command_class_expr: Option<Expr> = None;

    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("pid") {
            // Parses "pid = <expr>"
            pid_expr = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("command_class") {
            // Parses "command_class = <expr>"
            command_class_expr = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported property in rdm_parameter"))
        }
    });

    parse_macro_input!(args with parser);

    let pid = match pid_expr {
        Some(p) => p,
        None => {
            return syn::Error::new_spanned(name, "Missing required argument: 'pid'")
                .to_compile_error()
                .into();
        }
    };

    let command_class = match command_class_expr {
        Some(c) => c,
        None => {
            return syn::Error::new_spanned(name, "Missing required argument: 'command_class'")
                .to_compile_error()
                .into();
        }
    };

    let encode_steps = fields.iter().map(encode_step);

    let decode_steps = fields.iter().map(decode_step);

    let field_sizes = fields.iter().map(field_size);

    let expanded = quote! {
        // Emit the original struct
        #input

        impl rdm_core::parameter_traits::RdmParameterData for #name {
            fn size_of(&self) -> usize {
                0 #( + #field_sizes)*
            }

            fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, rdm_core::error::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_core::error::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                let mut offset = 0;

                #(#encode_steps)*

                Ok(offset)
            }

            fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, rdm_core::error::ParameterCodecError> {
                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }

        impl rdm_core::parameter_traits::RdmParameter for #name {
            const COMMAND_CLASS: rdm_core::CommandClass = #command_class;
            const PID: rdm_core::ParameterId = #pid;
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rdm_response_parameter(args: TokenStream, input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = &input.ident;

    let fields = match input.clone().data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("RdmResponseParameterCodec only supports named fields"),
        },
        _ => panic!("Only structs supported"),
    };

    let mut pid_expr: Option<Expr> = None;
    let mut command_class_expr: Option<Expr> = None;

    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("pid") {
            // Parses "pid = <expr>"
            pid_expr = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("command_class") {
            // Parses "command_class = <expr>"
            command_class_expr = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported property in rdm_parameter"))
        }
    });

    parse_macro_input!(args with parser);

    let pid = match pid_expr {
        Some(p) => p,
        None => {
            return syn::Error::new_spanned(name, "Missing required argument: 'pid'")
                .to_compile_error()
                .into();
        }
    };

    let command_class = match command_class_expr {
        Some(c) => c,
        None => {
            return syn::Error::new_spanned(name, "Missing required argument: 'command_class'")
                .to_compile_error()
                .into();
        }
    };

    let encode_steps = fields.iter().map(encode_step);

    let decode_steps = fields.iter().map(decode_step);

    let field_sizes = fields.iter().map(field_size);

    let expanded = quote! {
        // Emit the original struct
        #input

        impl rdm_core::parameter_traits::RdmParameterData for #name {
            fn size_of(&self) -> usize {
                0 #( + #field_sizes)*
            }

            fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, rdm_core::error::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_core::error::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                let mut offset = 0;

                #(#encode_steps)*

                Ok(offset)
            }

            fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, rdm_core::error::ParameterCodecError> {
                let mut offset = 0;

                Ok(Self {
                    #(#decode_steps),*
                })
            }
        }

        impl rdm_core::parameter_traits::RdmParameter for #name {
            const COMMAND_CLASS: rdm_core::CommandClass = #command_class;
            const PID: rdm_core::ParameterId = #pid;
        }
    };

    TokenStream::from(expanded)
}
