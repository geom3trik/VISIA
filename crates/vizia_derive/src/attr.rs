// Adapted from Druid attr.rs

// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{parenthesized, Error, ExprPath, Lit, LitStr};

const BASE_DATA_ATTR_PATH: &str = "data";
const BASE_LENS_ATTR_PATH: &str = "lens";
const IGNORE_ATTR_PATH: &str = "ignore";
const DATA_SAME_FN_ATTR_PATH: &str = "same_fn";
const DATA_EQ_ATTR_PATH: &str = "eq";
const LENS_NAME_OVERRIDE_ATTR_PATH: &str = "name";

/// The fields for a struct or an enum variant.
pub struct Fields<Attrs> {
    pub kind: FieldKind,
    fields: Vec<Field<Attrs>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    Named,
    // this also covers Unit; we determine 'unit-ness' based on the number
    // of fields.
    Unnamed,
}

#[derive(Debug)]
pub enum FieldIdent {
    Named(String),
    Unnamed(usize),
}

impl FieldIdent {
    pub fn unwrap_named(&self) -> syn::Ident {
        if let FieldIdent::Named(s) = self {
            syn::Ident::new(s, Span::call_site())
        } else {
            panic!("Unwrap named called on unnamed FieldIdent");
        }
    }
}

pub struct Field<Attrs> {
    pub ident: FieldIdent,
    pub ty: syn::Type,
    pub vis: syn::Visibility,
    pub attrs: Attrs,
}

pub enum DataAttr {
    Empty,
    Ignore,
    SameFn(ExprPath),
    Eq,
}

impl PartialEq for DataAttr {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (DataAttr::Empty, DataAttr::Empty)
                | (DataAttr::Ignore, DataAttr::Ignore)
                | (DataAttr::SameFn(_), DataAttr::SameFn(_))
                | (DataAttr::Eq, DataAttr::Eq)
        )
    }
}

#[derive(Debug)]
pub struct LensAttrs {
    /// `true` if this field should be ignored.
    pub ignore: bool,
    pub lens_name_override: Option<Ident>,
}

impl Fields<DataAttr> {
    pub fn parse_ast(fields: &syn::Fields) -> Result<Self, Error> {
        let kind = match fields {
            syn::Fields::Named(_) => FieldKind::Named,
            syn::Fields::Unnamed(_) | syn::Fields::Unit => FieldKind::Unnamed,
        };

        let fields = fields
            .iter()
            .enumerate()
            .map(|(i, field)| Field::<DataAttr>::parse_ast(field, i))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Fields { kind, fields })
    }
}

impl Fields<LensAttrs> {
    pub fn parse_ast(fields: &syn::Fields) -> Result<Self, Error> {
        let kind = match fields {
            syn::Fields::Named(_) => FieldKind::Named,
            syn::Fields::Unnamed(_) | syn::Fields::Unit => FieldKind::Unnamed,
        };

        let fields = fields
            .iter()
            .enumerate()
            .map(|(i, field)| Field::<LensAttrs>::parse_ast(field, i))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Fields { kind, fields })
    }
}

impl<Attrs> Fields<Attrs> {
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Field<Attrs>> {
        self.fields.iter()
    }
}

impl Field<DataAttr> {
    pub fn parse_ast(field: &syn::Field, index: usize) -> Result<Self, Error> {
        let ident = match field.ident.as_ref() {
            Some(ident) => FieldIdent::Named(ident.to_string().trim_start_matches("r#").to_owned()),
            None => FieldIdent::Unnamed(index),
        };

        let ty = field.ty.clone();

        let vis = field.vis.clone();

        let mut data_attr = DataAttr::Empty;
        for attr in field.attrs.iter() {
            if attr.path().is_ident(BASE_DATA_ATTR_PATH) {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident(IGNORE_ATTR_PATH) {
                        data_attr = DataAttr::Ignore;
                        return Ok(());
                    }

                    if meta.path.is_ident(DATA_EQ_ATTR_PATH) {
                        data_attr = DataAttr::Eq;
                        return Ok(());
                    }

                    if meta.path.is_ident(DATA_SAME_FN_ATTR_PATH) {
                        let content;
                        parenthesized!(content in meta.input);
                        let lit: LitStr = content.parse()?;
                        let expr = parse_lit_into_expr_path(&Lit::Str(lit))?;
                        data_attr = DataAttr::SameFn(expr);
                        return Ok(());
                    }

                    Err(Error::new(
                        meta.input.span(),
                        "Expected attribute list of the form #[data(one, two)]",
                    ))
                })?;
            }
        }
        Ok(Field { ident, ty, vis, attrs: data_attr })
    }

    pub fn same_fn_path_tokens(&self) -> TokenStream {
        match &self.attrs {
            DataAttr::SameFn(f) => quote!(#f),
            DataAttr::Eq => quote!(::core::cmp::PartialEq::eq),
            // this should not be called for DataAttr::Ignore
            DataAttr::Ignore => quote!(compiler_error!),
            DataAttr::Empty => {
                let span = Span::call_site();
                quote_spanned!(span=> Data::same)
            }
        }
    }
}

impl Field<LensAttrs> {
    pub fn parse_ast(field: &syn::Field, index: usize) -> Result<Self, Error> {
        let ident = match field.ident.as_ref() {
            Some(ident) => FieldIdent::Named(ident.to_string().trim_start_matches("r#").to_owned()),
            None => FieldIdent::Unnamed(index),
        };

        let ty = field.ty.clone();

        let vis = field.vis.clone();

        let mut ignore = false;
        let mut lens_name_override = None;

        for attr in field.attrs.iter() {
            if attr.path().is_ident(BASE_LENS_ATTR_PATH) {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident(IGNORE_ATTR_PATH) {
                        if ignore {
                            return Err(Error::new(meta.input.span(), "Duplicate attribute"));
                        }

                        ignore = true;
                        return Ok(());
                    }

                    if meta.path.is_ident(LENS_NAME_OVERRIDE_ATTR_PATH) {
                        if lens_name_override.is_some() {
                            return Err(Error::new(meta.input.span(), "Duplicate attribute"));
                        }

                        let content;
                        parenthesized!(content in meta.input);
                        let lit: LitStr = content.parse()?;
                        let ident = parse_lit_into_ident(&Lit::Str(lit))?;
                        lens_name_override = Some(ident);
                        return Ok(());
                    }

                    Err(Error::new(
                        meta.input.span(),
                        "Expected attribute list of the form #[lens(one, two)]",
                    ))
                })?;
            }
        }
        Ok(Field { ident, ty, vis, attrs: LensAttrs { ignore, lens_name_override } })
    }
}

impl<Attrs> Field<Attrs> {
    pub fn ident_tokens(&self) -> TokenTree {
        match self.ident {
            FieldIdent::Named(ref s) => Ident::new(s, Span::call_site()).into(),
            FieldIdent::Unnamed(num) => Literal::usize_unsuffixed(num).into(),
        }
    }

    pub fn ident_string(&self) -> String {
        match self.ident {
            FieldIdent::Named(ref s) => s.clone(),
            FieldIdent::Unnamed(num) => num.to_string(),
        }
    }
}

fn parse_lit_into_expr_path(lit: &syn::Lit) -> Result<ExprPath, Error> {
    let string = if let syn::Lit::Str(lit) = lit {
        lit
    } else {
        return Err(Error::new(lit.span(), "expected str, found... something else"));
    };

    let tokens = syn::parse_str(&string.value())?;
    syn::parse2(tokens)
}

fn parse_lit_into_ident(lit: &syn::Lit) -> Result<Ident, Error> {
    let ident = if let syn::Lit::Str(lit) = lit {
        Ident::new(&lit.value(), lit.span())
    } else {
        return Err(Error::new(lit.span(), "expected str, found... something else"));
    };

    Ok(ident)
}
