use std::collections::HashSet;
use std::fs;
use std::iter;
use std::path::PathBuf;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::ToTokens;

use crate::errors::CargoPlayError;

const USE_KEYWORDS: &'static [&'static str] = &["std", "core", "crate", "self", "alloc", "super"];

fn extra_use<'a, T: 'a + IntoIterator<Item = TokenTree> + Clone>(
    input: T,
) -> Box<dyn Iterator<Item = Ident> + 'a> {
    use proc_macro2::Literal;
    use TokenTree as tt;

    Box::new(
        input
            .clone()
            .into_iter()
            .zip(
                input
                    .into_iter()
                    .skip(1)
                    .chain(iter::once(Literal::u8_suffixed(1u8).into())),
            )
            .flat_map(|(prev, current)| match (prev, current) {
                (tt::Ident(ref first), tt::Ident(ref second)) if first.to_string() == "use" => {
                    Box::new(iter::once(second.clone()))
                }
                (tt::Group(ref group), _) => extra_use(group.stream()),
                _ => Box::new(iter::empty()),
            }),
    )
}

pub fn analyze_sources(sources: &Vec<PathBuf>) -> Result<HashSet<String>, CargoPlayError> {
    let contents: Vec<_> = sources
        .into_iter()
        .map(fs::read_to_string)
        .collect::<Result<_, _>>()?;

    let streams: Vec<TokenStream> = contents
        .into_iter()
        .map(|file| -> Result<_, CargoPlayError> {
            Ok(syn::parse_file(&file)?.into_token_stream())
        })
        .collect::<Result<_, CargoPlayError>>()?;

    Ok(streams
        .into_iter()
        .flat_map(|token| extra_use(token.into_iter()))
        .map(|ident| ident.to_string())
        .filter(|ident| !USE_KEYWORDS.contains(&ident.as_ref()))
        .collect())
}
