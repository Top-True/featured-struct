use crate::parse::FeatureName;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::Field;
use syn::Generics;
use syn::Ident;
use syn::{Error as SynError, Result as SynResult};

#[inline]
fn summon_getter(fields: &Vec<Field>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let mut_field_ident = Ident::new(
                format!("mut_{}", field_ident.to_string()).as_str(),
                field_ident.span(),
            );
            let ty = &field.ty;
            quote! {
                fn #field_ident(&self) -> &#ty {
                    &self.#field_ident
                }
                fn #mut_field_ident(&mut self) -> &mut #ty {
                    &mut self.#field_ident
                }
            }
        })
        .collect()
}

pub struct WithIndex<'a> {
    data: HashMap<&'a FeatureName, usize>,
}

impl<'a> WithIndex<'a> {
    pub fn new(data: impl Iterator<Item = &'a FeatureName>) -> Self {
        Self {
            data: data.enumerate().map(|(i, n)| (n, i)).collect(),
        }
    }

    pub fn query(&self, name: &FeatureName) -> Option<usize> {
        self.data.get(name).cloned()
    }

    pub fn query_as_ident(&self, name: &FeatureName) -> SynResult<Ident> {
        let Some(ident) = self.query(name) else {
            return Err(SynError::new(name.span(), "Unknown feature name"));
        };
        Ok(Ident::new(format!("r#F{}", ident).as_str(), name.span()))
    }

    pub fn summon_with(
        &self,
        generics: &'a Generics,
        common_fields: &'a Vec<Field>,
        featured_fields: &'a HashMap<FeatureName, Vec<Field>>,
    ) -> SynResult<TokenStream> {
        let common_methods = summon_getter(common_fields);
        let mut featured_with = Vec::with_capacity(featured_fields.len());
        for (name, fields) in featured_fields.iter() {
            let ident = self.query_as_ident(name)?;
            let methods = summon_getter(fields);
            featured_with.push(quote! {
                pub trait #ident #generics {
                    #(#methods)*
                }
            });
        }
        Ok(quote! {
            pub mod with {
                use super::*;
                pub trait Common #generics {
                    #(#common_methods)*
                }
                #(#featured_with)*
            }
        })
    }
}
