pub fn is_bool(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        tp.path.is_ident("bool")
    } else {
        false
    }
}

pub fn is_u8(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        tp.path.is_ident("u8")
    } else {
        false
    }
}
