use std::collections::HashSet;
use std::iter;

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

pub fn analyze_sources(
    stdin: Option<&str>,
    sources: &[&str],
) -> Result<HashSet<String>, CargoPlayError> {
    let streams: Vec<TokenStream> = stdin
        .iter()
        .chain(sources.iter())
        .map(|source| -> Result<_, CargoPlayError> {
            Ok(syn::parse_file(source)?.into_token_stream())
        })
        .collect::<Result<_, CargoPlayError>>()?;

    Ok(streams
        .into_iter()
        .flat_map(|token| extra_use(token.into_iter()))
        .map(|ident| ident.to_string())
        .filter(|ident| !USE_KEYWORDS.contains(&ident.as_ref()))
        .collect())
}
