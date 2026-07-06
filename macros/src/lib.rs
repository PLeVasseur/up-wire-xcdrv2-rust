/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

#[proc_macro_derive(XcdrV2Type, attributes(xcdr_v2, xcdrv2))]
pub fn derive_xcdr_v2_type(input: TokenStream) -> TokenStream {
    expand_xcdr_v2_type(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_xcdr_v2_type(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let type_name = type_name(&input)?;
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| {
                    Ok((
                        field
                            .ident
                            .as_ref()
                            .expect("named fields have identifiers")
                            .clone(),
                        field.ty.clone(),
                    ))
                })
                .collect::<syn::Result<Vec<_>>>()?,
            Fields::Unit => Vec::new(),
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "XcdrV2Type derive supports named-field structs and unit structs only",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "XcdrV2Type derive supports structs only",
            ))
        }
    };

    let field_idents = fields.iter().map(|(ident, _)| ident).collect::<Vec<_>>();
    let field_tys = fields.iter().map(|(_, ty)| ty).collect::<Vec<_>>();
    let mut generics = input.generics.clone();
    {
        let where_clause = generics.make_where_clause();
        for ty in &field_tys {
            where_clause
                .predicates
                .push(syn::parse_quote!(#ty: ::up_wire_xcdrv2::XcdrV2Field));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::up_wire_xcdrv2::XcdrV2Struct for #name #ty_generics #where_clause {
            const TYPE_NAME: &'static str = #type_name;

            fn encode_xcdr_v2_fields(
                &self,
                encoder: &mut ::up_wire_xcdrv2::XcdrV2Encoder,
            ) -> Result<(), ::up_rust::UWireError> {
                #(
                    ::up_wire_xcdrv2::XcdrV2Field::encode_field(&self.#field_idents, encoder)?;
                )*
                Ok(())
            }

            fn decode_xcdr_v2_fields(
                decoder: &mut ::up_wire_xcdrv2::XcdrV2Decoder<'_>,
            ) -> Result<Self, ::up_rust::UWireError> {
                Ok(Self {
                    #(
                        #field_idents: <#field_tys as ::up_wire_xcdrv2::XcdrV2Field>::decode_field(decoder)?,
                    )*
                })
            }
        }
    })
}

fn type_name(input: &DeriveInput) -> syn::Result<String> {
    for attr in &input.attrs {
        if attr.path().is_ident("xcdr_v2") || attr.path().is_ident("xcdrv2") {
            let mut configured = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("type_name") {
                    let value: LitStr = meta.value()?.parse()?;
                    configured = Some(value.value());
                    Ok(())
                } else {
                    Err(meta.error("unsupported XcdrV2Type attribute"))
                }
            })?;
            if let Some(configured) = configured {
                return Ok(configured);
            }
        }
    }

    Ok(input.ident.to_string())
}
