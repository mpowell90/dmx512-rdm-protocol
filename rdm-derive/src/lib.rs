use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Fields, Index, parse_macro_input};
extern crate self as rdm_derive;

fn get_field_accessor(f: &syn::Field, index: usize) -> proc_macro2::TokenStream {
    match &f.ident {
        Some(ident) => quote!(#ident),
        None => {
            let index = Index::from(index);
            quote!(#index)
        }
    }
}

fn field_size(pair: (usize, &syn::Field)) -> proc_macro2::TokenStream {
    let (i, f) = pair;
    let accessor = get_field_accessor(f, i);
    let ty = &f.ty;

    quote!(<#ty as rdm_core::parameter_traits::RdmParameterData>::size_of(&self.#accessor))
}

#[proc_macro_derive(RdmParameterData)]
pub fn derive_rdm_parameter_data(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident.clone();

    let fields_data = match input.data {
        Data::Struct(s) => s.fields,
        _ => panic!("Only structs supported"),
    };

    let encode_steps = fields_data.iter().enumerate().map(|(i, f)| {
        let accessor = get_field_accessor(f, i);
        let ty = &f.ty;

        quote! {
            offset += <#ty as rdm_core::parameter_traits::RdmParameterData>::encode_parameter_data(&self.#accessor, &mut buf[offset..])?;
        }
    });

    let decode_values = fields_data.iter().map(|f| {
        let ty = &f.ty;
        quote! {
            {
                let val = <#ty as rdm_core::parameter_traits::RdmParameterData>::decode_parameter_data(&buf[offset..])?;
                offset += <#ty as rdm_core::parameter_traits::RdmParameterData>::size_of(&val);
                val
            }
        }
    });

    let decode_self = match &fields_data {
        Fields::Named(fields) => {
            let names = fields.named.iter().map(|f| &f.ident);
            quote! {
                Self {
                    #( #names: #decode_values ),*
                }
            }
        }
        Fields::Unnamed(_) => {
            quote! {
                Self (
                    #( #decode_values ),*
                )
            }
        }
        Fields::Unit => quote!(Self),
    };

    let field_sizes = fields_data.iter().enumerate().map(field_size);

    let expanded = quote! {
        impl rdm_core::parameter_traits::RdmParameterData for #name {
            fn size_of(&self) -> usize {
                0 #( + #field_sizes)*
            }

            fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, rdm_core::error::ParameterCodecError> {
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

            fn decode_parameter_data(buf: &[u8]) -> Result<Self, rdm_core::error::ParameterCodecError> {
                let mut offset = 0;

                Ok(#decode_self)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rdm_request_parameter(args: TokenStream, input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = &input.ident;

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

    let expanded = quote! {
        #[derive(rdm_derive::RdmParameterData)]
        #input

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

    let expanded = quote! {
        #[derive(rdm_derive::RdmParameterData)]
        #input

        impl rdm_core::parameter_traits::RdmParameter for #name {
            const COMMAND_CLASS: rdm_core::CommandClass = #command_class;
            const PID: rdm_core::ParameterId = #pid;
        }
    };

    TokenStream::from(expanded)
}
