pub mod only_index;
pub mod with_index;

use crate::parse::{FeatureAnnotation, FeatureDeclarations, FeatureName};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashMap;
use syn::Field;
use syn::parse2;
use syn::spanned::Spanned;
use syn::{Error as SynError, Result as SynResult};
use syn::{Fields, FieldsNamed, Item};

#[inline]
fn extract_fields(
    fields: FieldsNamed,
    features_len: usize,
) -> (Vec<Field>, HashMap<FeatureName, Vec<Field>>) {
    let mut common_fields: Vec<Field> = Vec::with_capacity(fields.named.len());
    let mut featured_fields: HashMap<FeatureName, Vec<Field>> =
        HashMap::with_capacity(features_len);
    for mut field in fields.named {
        let mut feature_name = None;
        let mut attrs = Vec::with_capacity(field.attrs.len());
        for attr in field.attrs {
            if FeatureAnnotation::is_annotation(&attr) {
                feature_name = Some(
                    parse2::<FeatureAnnotation>(attr.into_token_stream())
                        .unwrap()
                        .name,
                );
                continue;
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
    let Fields::Named(fields) = item.fields else {
        return Err(SynError::new(item.span(), "need named fields"));
    };
    let combinations = declarations.combinations();
    let item_name = item.ident;
    let vis = item.vis;
    let generics = item.generics;
    let (common_fields, featured_fields) = extract_fields(
        fields,
        declarations.units.len() + declarations.compositions.len(),
    );
    let with_index = with_index::WithIndex::new(
        declarations
            .units
            .iter()
            .chain(declarations.compositions.keys()),
    );
    let with_quote = with_index.summon_with(&generics, &common_fields, &featured_fields)?;

    Ok(quote! {
        #[allow(non_snake_case)]
        #vis mod #item_name {
            use super::*;
            pub mod __private {
                use super::*;
                #with_quote
                pub mod only {
                    use super::*;

                }
            }

            /* summon */
        }
    })
}
