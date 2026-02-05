use crate::parse::{FeatureAnnotation, FeatureDeclarations, FeatureName};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashMap;
use syn::Field;
use syn::parse2;
use syn::spanned::Spanned;
use syn::{Error as SynError, Result as SynResult};
use syn::{Fields, Item};

#[inline]
fn extract_fields(
    fields: Fields,
    features_len: usize,
) -> (Vec<Field>, HashMap<FeatureName, Vec<Field>>) {
    let mut common_fields: Vec<Field> = Vec::with_capacity(fields.len());
    let mut featured_fields: HashMap<FeatureName, Vec<Field>> =
        HashMap::with_capacity(features_len);
    for mut field in fields {
        let mut feature_name = None;
        let mut attrs = Vec::with_capacity(field.attrs.len());
        for attr in field.attrs {
            if FeatureAnnotation::is_annotation(&attr) {
                feature_name = Some(
                    parse2::<FeatureAnnotation>(attr.into_token_stream())
                        .unwrap()
                        .name,
                );
                break;
            }
            attrs.push(attr);
        }
        field.attrs = attrs;
        if let Some(feature_name) = feature_name {
            match featured_fields.get_mut(&feature_name) {
                Some(v) => v.push(field),
                None => {
                    featured_fields.insert(feature_name, vec![field]);
                }
            }
        } else {
            common_fields.push(field);
        }
    }
    (common_fields, featured_fields)
}

pub fn summon(declarations: FeatureDeclarations, item: Item) -> SynResult<TokenStream> {
    let Item::Struct(item) = item else {
        return Err(SynError::new(item.span(), "not a struct"));
    };
    let combinations = declarations.combinations();
    let item_name = item.ident;
    let vis = item.vis;
    let generics = item.generics;
    let (common_fields, featured_fields) = extract_fields(item.fields, combinations.len());
    // todo
    quote! {
        pub mod r#f0 {
            pub struct Only #generics {

            }

            pub trait With #generics {
                
            }
        }
    };

    Ok(quote! {
        #[allow(non_snake_case)]
        #vis mod #item_name {
            use super::*;

            pub mod __private {
                use super::*;

                /* summon */
            }

            /* summon */
        }
    })
}
