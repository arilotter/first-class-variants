extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, AttributeArgs, Field, Fields, ItemEnum, Lit,
    Meta, MetaNameValue, NestedMeta, Token, VisPublic,
};

#[proc_macro_attribute]
pub fn first_class_variants(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as AttributeArgs);
    let input = parse_macro_input!(item as ItemEnum);
    let vis = &input.vis;
    let name = &input.ident;
    let enum_attrs = &input.attrs;
    let variants = &input.variants;

    // parse module="..." and prefix="..." params if present
    // and filter them out from the attributes to pass through
    let mut mod_name = None;
    let mut custom_prefix = None;
    let mut passthrough_attrs = Vec::new();

    for arg in &attr_args {
        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) = arg {
            if path.is_ident("module") {
                if let Lit::Str(s) = lit {
                    mod_name = Some(Ident::new(&s.value(), Span::call_site()));
                }
                continue; // skip it
            } else if path.is_ident("prefix") {
                if let Lit::Str(s) = lit {
                    custom_prefix = Some(s.value());
                }
                continue; // skip it
            }
        }
        passthrough_attrs.push(arg);
    }

    // prefix struct?
    let default_prefix = name.to_string();
    let prefix = custom_prefix.as_ref().unwrap_or(&default_prefix);

    let make_struct_ident = |variant_ident: &Ident| {
        if mod_name.is_some() {
            // if we use mod, don't prefix
            variant_ident.clone()
        } else {
            // use prefix otherwise
            Ident::new(
                &format!("{}{}", prefix, variant_ident.to_string()),
                Span::call_site(),
            )
        }
    };
    let variant_structs = variants.iter().map(|v| {
        let variant_ident = &v.ident;
        let variant_attrs = &v.attrs;
        let struct_ident = make_struct_ident(variant_ident);
        let mut fields = v.fields.clone();
        match &mut fields {
            Fields::Named(named) => make_pub(&mut named.named),
            Fields::Unnamed(unnamed) => make_pub(&mut unnamed.unnamed),
            _ => {}
        }

        let semicolon = match &v.fields {
            Fields::Named(_) => None,
            _ => Some(<Token!(;)>::default()),
        };
        quote! {
            #(
                #[#passthrough_attrs]
            )*
            #(#variant_attrs)*
            pub struct #struct_ident #fields #semicolon
            impl From<#struct_ident> for #name {
                fn from(subtype_struct: #struct_ident) -> Self {
                    #name::#variant_ident(subtype_struct)
                }
            }
            impl std::convert::TryFrom<#name> for #struct_ident {
                type Error = (); // There's only one possible error - enum variant isn't this struct.
                fn try_from(enum_variant: #name) -> Result<Self, Self::Error> {
                    match enum_variant {
                        #name::#variant_ident(subtype_struct) => Ok(subtype_struct),
                        _ => Err(())
                    }
                }
            }
            impl<'a> std::convert::TryFrom<&'a #name> for &'a #struct_ident {
                type Error = (); // There's only one possible error - enum variant isn't this struct.
                fn try_from(enum_variant: &'a #name) -> Result<Self, Self::Error> {
                    match enum_variant {
                        #name::#variant_ident(subtype_struct) => Ok(subtype_struct),
                        _ => Err(())
                    }
                }
            }
        }
    });
    let wrapper_variants = variants.iter().map(|v| {
        let variant_ident = &v.ident;
        let struct_ident = make_struct_ident(variant_ident);

        if let Some(ref mod_ident) = mod_name {
            quote! {
                #variant_ident(#mod_ident::#struct_ident)
            }
        } else {
            quote! {
                #variant_ident(#struct_ident)
            }
        }
    });

    let result = if let Some(ref mod_ident) = mod_name {
        // wrap in a module
        quote! {
            #(#enum_attrs)*
            #vis enum #name {
                #(#wrapper_variants,)*
            }
            #vis mod #mod_ident {
                use super::*;
                #(#variant_structs)*
            }
        }
    } else {
        quote! {
            #(#enum_attrs)*
            #vis enum #name {
                #(#wrapper_variants,)*
            }
            #(#variant_structs)*
        }
    };
    result.into()
}

fn make_pub(punctuated: &mut Punctuated<Field, Token![,]>) {
    for field in punctuated.iter_mut() {
        field.vis = VisPublic {
            pub_token: <Token![pub]>::default(),
        }
        .into();
    }
}
