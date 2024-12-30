use fluent_templates::{fluent_bundle::FluentValue, LanguageIdentifier, Loader};
use std::borrow::Cow;

fluent_templates::static_loader! {
    pub static LOCALES = {
        locales: ".\\locales",
        fallback_language: "en-US",
    };
}

pub fn fluent<T: std::fmt::Display>(key: T, lang: &LanguageIdentifier) -> ::rinja::Result<String> {
    let key = key.to_string();
    Ok(LOCALES.lookup(lang, &key))
}

pub fn fluent_args<T: std::fmt::Display>(
    key: T,
    lang: &LanguageIdentifier,
    args: std::collections::HashMap<&str, String>,
) -> ::rinja::Result<String> {
    let key = key.to_string();
    let fluent_args: std::collections::HashMap<Cow<'static, str>, FluentValue<'static>> = args
        .into_iter()
        .map(|(k, v)| (k.to_owned().into(), FluentValue::String(v.into())))
        .collect();

    Ok(LOCALES.lookup_with_args(lang, &key, &fluent_args))
}
