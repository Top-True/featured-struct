use itertools::Itertools;
use proc_macro2::Span;
use std::collections::HashMap;
use std::fmt::Formatter;
use syn::Result as ParseResult;
use syn::Token;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error as ParseError, bracketed};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct FeatureName(syn::Ident);

impl FeatureName {
    pub fn span(&self) -> Span {
        self.0.span()
    }
}

impl Parse for FeatureName {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(FeatureName(input.parse()?))
    }
}

#[derive(Debug, Clone)]
pub struct FeaturesCompositionExpression {
    pub enables: Vec<FeatureName>,
    pub disables: Vec<FeatureName>,
}

impl Parse for FeaturesCompositionExpression {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut enables = Vec::with_capacity(2);
        let mut disables = Vec::with_capacity(2);
        loop {
            let is_disable = input.parse::<Token![!]>().is_ok();
            let feature = input.parse::<FeatureName>()?;
            if is_disable {
                disables.push(feature);
            } else {
                enables.push(feature);
            }
            if input.peek(Token![+]) {
                continue;
            } else {
                break;
            }
        }
        Ok(Self { enables, disables })
    }
}

impl FeaturesCompositionExpression {
    pub fn will_enabled(&self, enableds: &[&FeatureName]) -> bool {
        for disable in self.disables.iter() {
            if enableds.contains(&disable) {
                return false;
            }
        }
        for enable in self.enables.iter() {
            if !enableds.contains(&enable) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct FeatureDeclarations {
    pub units: Vec<FeatureName>,
    pub compositions: HashMap<FeatureName, FeaturesCompositionExpression>,
}

impl Parse for FeatureDeclarations {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        enum Declaration {
            Unit(FeatureName),
            Composition(FeatureName, FeaturesCompositionExpression),
        }
        impl Parse for Declaration {
            fn parse(input: ParseStream) -> ParseResult<Self> {
                let name = input.parse::<FeatureName>()?;
                if input.parse::<Token![=]>().is_ok() {
                    Ok(Declaration::Composition(name, input.parse()?))
                } else {
                    Ok(Declaration::Unit(name))
                }
            }
        }

        let mut units = Vec::with_capacity(1);
        let mut compositions = HashMap::new();
        for d in Punctuated::<Declaration, Token![,]>::parse_terminated(input)? {
            match d {
                Declaration::Unit(n) => {
                    units.push(n);
                }
                Declaration::Composition(n, e) => {
                    compositions.insert(n, e);
                }
            }
        }
        match (Self {
            units,
            compositions,
        })
        .expand()
        {
            Ok(x) => Ok(x),
            Err(e) => Err(ParseError::new(input.span(), e)),
        }
    }
}

impl FeatureDeclarations {
    pub fn combinations(&self) -> Vec<Vec<&FeatureName>> {
        let mut result = Vec::with_capacity(32);
        for i in 1..=self.units.len() {
            result.extend(self.units.iter().combinations(i).collect::<Vec<_>>());
        }
        for c in result.iter_mut() {
            for (n, e) in self.compositions.iter() {
                if e.will_enabled(c) {
                    c.push(n);
                }
            }
        }
        result
    }
}

impl FeatureDeclarations {
    fn expand(mut self) -> Result<FeatureDeclarations, ExpandError> {
        let mut sorted: Vec<FeatureName> = Vec::with_capacity(self.compositions.len());
        let mut com = self
            .compositions
            .iter()
            .map(|(n, e)| {
                (
                    n,
                    e.enables
                        .iter()
                        .chain(e.disables.iter())
                        .filter(|x| !self.units.contains(x))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let mut com2 = Vec::with_capacity(com.len());
        let mut prev_count;
        while !com.is_empty() {
            prev_count = com.len();
            while let Some((name, dependencies)) = com.pop() {
                let dependencies = dependencies
                    .into_iter()
                    .filter(|x| !sorted.contains(x))
                    .collect::<Vec<_>>();
                if dependencies.is_empty() {
                    sorted.push(name.clone());
                } else {
                    com2.push((name, dependencies));
                }
            }
            if com2.len() == prev_count {
                return Err(ExpandError::UndeclaredFeature);
            }
            com = com2.drain(..).collect::<Vec<_>>();
        }
        let units = self.units;
        let mut compositions: HashMap<FeatureName, FeaturesCompositionExpression> =
            HashMap::with_capacity(self.compositions.len());
        for f in sorted {
            let mut expr = FeaturesCompositionExpression {
                enables: Vec::new(),
                disables: Vec::new(),
            };
            let prev = self.compositions.remove(&f).unwrap();
            for e in prev.enables {
                if units.contains(&e) {
                    expr.enables.push(e);
                } else {
                    expr.enables
                        .extend(compositions.get(&e).unwrap().enables.clone());
                }
            }
            for d in prev.disables {
                if units.contains(&d) {
                    expr.disables.push(d);
                } else {
                    expr.disables
                        .extend(compositions.get(&d).unwrap().enables.clone());
                }
            }
            compositions.insert(f, expr);
        }
        Ok(FeatureDeclarations {
            units,
            compositions,
        })
    }
}

#[derive(Debug)]
pub enum ExpandError {
    UndeclaredFeature,
}

impl std::fmt::Display for ExpandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for ExpandError {}

pub struct FeatureAnnotation {
    pub name: FeatureName,
}

impl FeatureAnnotation {
    pub fn is_annotation(attr: &syn::Attribute) -> bool {
        matches!(attr.style, syn::AttrStyle::Outer)
            && attr.meta.require_list().is_ok_and(|x| {
                let Some(x) = x.tokens.clone().into_iter().next() else {
                    return false;
                };
                let proc_macro2::TokenTree::Ident(ident) = x else {
                    return false;
                };
                ident.to_string() == "featruct"
            })
    }
}

impl Parse for FeatureAnnotation {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        input.parse::<Token![#]>()?;
        let content;
        bracketed!(content in input);
        if !(content.parse::<syn::Ident>()?.to_string() == "featruct") {
            return Err(ParseError::new(input.span(), "Not a feature annotation"));
        };
        Ok(FeatureAnnotation {
            name: content.parse()?,
        })
    }
}
