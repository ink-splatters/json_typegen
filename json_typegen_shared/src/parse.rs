//! Functions for parsing `json_typegen` macro invocations and their arguments
//!
//! Requires the "option-parsing" feature

use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Ident, LitBool, LitStr, Token, braced, parenthesized};

use crate::hints::Hint;
use crate::options::{ImportStyle, InputMode, Options, OutputMode, StringTransform};

#[derive(PartialEq, Debug)]
pub struct MacroInput {
    pub name: String,
    pub sample_source: String,
    pub options: Options,
}

mod keyword {
    syn::custom_keyword!(json_typegen);
}

struct FullMacro(MacroInput);

impl Parse for FullMacro {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<keyword::json_typegen>()?;
        input.parse::<Token![!]>()?;

        let content;
        parenthesized!(content in input);
        let macro_input = content.parse()?;

        input.parse::<Token![;]>()?;

        Ok(Self(macro_input))
    }
}

impl Parse for MacroInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = parse_string(input, "First argument must be a string literal")?;

        parse_comma(input, "Expected a comma after first argument")?;

        let sample_source = parse_string(input, "Second argument must be a string literal")?;
        let default_options = Options::macro_default();

        let options = if input.is_empty() {
            default_options
        } else {
            parse_comma(
                input,
                "Expected a comma or end of input after second argument",
            )?;

            if input.peek(LitStr) {
                let options: LitStr = input.parse()?;
                options.parse_with(move |input: ParseStream<'_>| {
                    parse_options_block(input, default_options)
                })?
            } else {
                parse_options_block(input, default_options)?
            }
        };

        if !input.is_empty() {
            return Err(input.error("Expected no further tokens after options block"));
        }

        Ok(Self {
            name,
            sample_source,
            options,
        })
    }
}

/// Parses a full `json_typegen` macro invocation. E.g. something like
/// `json_typegen!("Foo", "http://example.com/sample.json", { deny_unknown_fields });`
pub fn full_macro(input: &str) -> Result<MacroInput, String> {
    syn::parse_str::<FullMacro>(input)
        .map(|input| input.0)
        .map_err(|error| error.to_string())
}

/// Parses the arguments to a `json_typegen` macro invocation. E.g. something like
/// `"Foo", "http://example.com/sample.json", { deny_unknown_fields }`
pub fn macro_input(input: &str) -> Result<MacroInput, String> {
    syn::parse_str(input).map_err(|error| error.to_string())
}

/// Parses the options block of a `json_typegen` macro invocation. E.g. something like:
/// `{ deny_unknown_fields }`
pub fn options(input: &str) -> Result<Options, String> {
    options_with_defaults(input, Options::default())
}

fn options_with_defaults(input: &str, default_options: Options) -> Result<Options, String> {
    (move |input: ParseStream<'_>| parse_options_block(input, default_options))
        .parse_str(input)
        .map_err(|error| error.to_string())
}

fn parse_options_block(input: ParseStream<'_>, mut options: Options) -> syn::Result<Options> {
    let content;
    braced!(content in input);

    while !content.is_empty() {
        let option_name = parse_option_name(&content)?;

        match option_name.as_str() {
            "output_mode" => {
                let value = string_option(&content, "output_mode")?;
                options.output_mode = OutputMode::parse(&value).unwrap_or(OutputMode::Rust);
            }
            "input_mode" => {
                let value = string_option(&content, "input_mode")?;
                options.input_mode = InputMode::parse(&value).unwrap_or(InputMode::Json);
            }
            "derives" => options.derives = string_option(&content, "derives")?,
            "property_name_format" => {
                let value = string_option(&content, "property_name_format")?;
                options.property_name_format = StringTransform::parse(&value);
            }
            "import_style" => {
                let value = string_option(&content, "import_style")?;
                options.import_style =
                    ImportStyle::parse(&value).unwrap_or(ImportStyle::QualifiedPaths);
            }
            "field_visibility" => {
                options.field_visibility = Some(string_option(&content, "field_visibility")?);
            }
            "deny_unknown_fields" => {
                options.deny_unknown_fields = boolean_option(&content, "deny_unknown_fields")?;
            }
            "use_default_for_missing_fields" => {
                options.use_default_for_missing_fields =
                    boolean_option(&content, "use_default_for_missing_fields")?;
            }
            "allow_option_vec" => {
                options.allow_option_vec = boolean_option(&content, "allow_option_vec")?;
            }
            "collect_additional" => {
                options.collect_additional = boolean_option(&content, "collect_additional")?;
            }
            "unwrap" => options.unwrap = string_option(&content, "unwrap")?,
            "infer_map_threshold" => {
                let value = string_option(&content, "infer_map_threshold")?;
                options.infer_map_threshold = value.parse().ok();
            }
            key if key.is_empty() || key.starts_with('/') => {
                for hint in pointer_block(&content)? {
                    options.hints.push((key.to_owned(), hint));
                }
            }
            _ => return Err(content.error(format!("Unknown option: {option_name}"))),
        }

        parse_separator(&content)?;
    }

    Ok(options)
}

fn pointer_block(input: ParseStream<'_>) -> syn::Result<Vec<Hint>> {
    let mut hints = Vec::new();

    parse_colon(input)?;
    let content;
    braced!(content in input);

    while !content.is_empty() {
        let key = parse_option_name(&content)?;

        match key.as_str() {
            "use_type" => {
                let value = string_option(&content, "use_type")?;
                hints.push(if value == "map" {
                    Hint::default_map()
                } else {
                    Hint::opaque_type(value)
                });
            }
            "type_name" => {
                let value = string_option(&content, "type_name")?;
                hints.push(Hint::type_name(value));
            }
            _ => return Err(content.error(format!("Unknown option: {key}"))),
        }

        parse_separator(&content)?;
    }

    Ok(hints)
}

fn parse_option_name(input: ParseStream<'_>) -> syn::Result<String> {
    if input.peek(LitStr) {
        input.parse::<LitStr>().map(|literal| literal.value())
    } else if input.peek(Ident::peek_any) {
        input.call(Ident::parse_any).map(|ident| ident.to_string())
    } else {
        Err(input.error("Expected an option name"))
    }
}

fn parse_string(input: ParseStream<'_>, message: &'static str) -> syn::Result<String> {
    if !input.peek(LitStr) {
        return Err(input.error(message));
    }

    input.parse::<LitStr>().map(|literal| literal.value())
}

fn string_option(input: ParseStream<'_>, name: &'static str) -> syn::Result<String> {
    parse_colon(input)?;

    if !input.peek(LitStr) {
        return Err(input.error(format!(
            "The argument to '{name}' has to be a string literal"
        )));
    }

    input.parse::<LitStr>().map(|literal| literal.value())
}

fn boolean_option(input: ParseStream<'_>, name: &'static str) -> syn::Result<bool> {
    // Interpret { foo, bar } as { foo: true, bar: true }.
    if input.is_empty() || input.peek(Token![,]) {
        return Ok(true);
    }

    parse_colon(input)?;

    if !input.peek(LitBool) {
        return Err(input.error(format!(
            "The argument to '{name}' has to be a boolean literal"
        )));
    }

    input.parse::<LitBool>().map(|literal| literal.value)
}

fn parse_comma(input: ParseStream<'_>, message: &'static str) -> syn::Result<()> {
    if !input.peek(Token![,]) {
        return Err(input.error(message));
    }

    input.parse::<Token![,]>().map(drop)
}

fn parse_separator(input: ParseStream<'_>) -> syn::Result<()> {
    if input.is_empty() {
        Ok(())
    } else {
        parse_comma(input, "Expected a comma or a closing brace")
    }
}

fn parse_colon(input: ParseStream<'_>) -> syn::Result<()> {
    if !input.peek(Token![:]) {
        return Err(input.error("Expected a colon"));
    }

    input.parse::<Token![:]>().map(drop)
}

#[cfg(test)]
mod macro_input_tests {
    use super::*;

    #[test]
    fn barebones_input() {
        assert_eq!(
            macro_input(r#" "Bob", "{}" "#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::macro_default(),
            })
        );
    }

    #[test]
    fn barebones_input_with_empty_options() {
        assert_eq!(
            macro_input(r#" "Bob", "{}", {} "#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::macro_default(),
            })
        );
    }

    #[test]
    fn barebones_input_with_options_as_string_literal() {
        assert_eq!(
            macro_input(r#" "Bob", "{}", "{}" "#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::macro_default(),
            })
        );
    }

    #[test]
    fn parses_raw_literals_and_string_encoded_options() {
        let parsed = macro_input(
            r##"r#"Bob"#, r#"{}"#, r#"{ deny_unknown_fields }"#"##,
        )
        .unwrap();

        assert_eq!(parsed.name, "Bob");
        assert_eq!(parsed.sample_source, "{}");
        assert!(parsed.options.deny_unknown_fields);
    }

    #[test]
    fn rejects_tokens_after_options() {
        assert!(macro_input(r#""Bob", "{}", "{}" trailing"#).is_err());
    }
}

#[cfg(test)]
mod options_tests {
    use super::*;

    #[test]
    fn parses_derives() {
        let expected = Options {
            derives: "Foo, Bar".into(),
            ..Options::default()
        };

        assert_eq!(
            options(
                r#"{
                "derives": "Foo, Bar",
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn rejects_unknown_options() {
        let result = options(
            r#"{
            "foo_opt": {},
        }"#,
        );

        assert!(
            result.is_err(),
            "Parse result was not Err, but:\n{:?}",
            result
        );
        if let Err(message) = result {
            assert!(
                message.contains("foo_opt"),
                "Error message was:\n'{}'",
                message
            );
        }
    }

    #[test]
    fn parses_empty_pointer_block() {
        let expected = Options::default();

        assert_eq!(
            options(
                r#"{
                "/foo/bar": {},
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_map_hint() {
        let mut expected = Options::default();
        expected
            .hints
            .push(("/foo/bar".to_string(), Hint::default_map()));

        assert_eq!(
            options(
                r#"{
                "/foo/bar": {
                    use_type: "map"
                },
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_opaque_type_hint() {
        let mut expected = Options::default();
        expected
            .hints
            .push(("/foo/bar".to_string(), Hint::opaque_type("FooBar")));

        assert_eq!(
            options(
                r#"{
                "/foo/bar": {
                    use_type: "FooBar"
                },
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_type_name_hint() {
        let mut expected = Options::default();
        expected
            .hints
            .push(("/baz".to_string(), Hint::type_name("SomeName")));

        assert_eq!(
            options(
                r#"{
                "/baz": {
                    type_name: "SomeName"
                },
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_multiple_hints_for_one_pointer() {
        let mut expected = Options::default();
        expected
            .hints
            .push(("/value".to_string(), Hint::opaque_type("Value")));
        expected
            .hints
            .push(("/value".to_string(), Hint::type_name("Renamed")));

        assert_eq!(
            options(
                r#"{
                    "/value": {
                        "use_type": "Value",
                        "type_name": "Renamed",
                    },
                }"#,
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_pointer_to_root() {
        let expected = Options::default();

        assert_eq!(
            options(
                r#"{
                "": {},
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_keys_given_as_bare_identifiers() {
        let expected = Options {
            derives: "Foo, Bar".into(),
            ..Options::default()
        };

        assert_eq!(
            options(
                r#"{
                derives: "Foo, Bar",
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn trailing_comma_is_optional() {
        let expected = Options {
            derives: "Foo, Bar".into(),
            ..Options::default()
        };

        assert_eq!(
            options(
                r#"{
                "derives": "Foo, Bar"
            }"#
            ),
            Ok(expected.clone())
        );

        assert_eq!(
            options(
                r#"{
                "derives": "Foo, Bar",
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_every_top_level_option_type() {
        let expected = Options {
            output_mode: OutputMode::Typescript,
            input_mode: InputMode::Sql,
            use_default_for_missing_fields: true,
            deny_unknown_fields: true,
            allow_option_vec: true,
            field_visibility: Some("pub(crate)".into()),
            derives: "Debug, Clone".into(),
            property_name_format: Some(StringTransform::SnakeCase),
            unwrap: "/data".into(),
            import_style: ImportStyle::AssumeExisting,
            collect_additional: true,
            infer_map_threshold: Some(12),
            ..Options::default()
        };

        assert_eq!(
            options(
                r#"{
                    output_mode: "typescript",
                    input_mode: "sql",
                    derives: "Debug, Clone",
                    property_name_format: "snake_case",
                    import_style: "assume_existing",
                    field_visibility: "pub(crate)",
                    deny_unknown_fields: true,
                    use_default_for_missing_fields,
                    allow_option_vec: true,
                    collect_additional,
                    unwrap: "/data",
                    infer_map_threshold: "12",
                }"#,
            ),
            Ok(expected)
        );
    }

    #[test]
    fn rejects_invalid_option_value_types() {
        let string_error = options("{ derives: true }").unwrap_err();
        assert!(string_error.contains("string literal"));

        let boolean_error = options(r#"{ deny_unknown_fields: "true" }"#).unwrap_err();
        assert!(boolean_error.contains("boolean literal"));
    }

    #[test]
    fn preserves_invalid_option_value_fallbacks() {
        let expected = Options {
            import_style: ImportStyle::QualifiedPaths,
            ..Options::default()
        };

        assert_eq!(
            options(
                r#"{
                    output_mode: "invalid",
                    input_mode: "invalid",
                    property_name_format: "invalid",
                    import_style: "invalid",
                    infer_map_threshold: "invalid",
                }"#,
            ),
            Ok(expected)
        );
    }
}

#[cfg(test)]
mod full_macro_tests {
    use super::*;

    #[test]
    fn full_macro_accepts_barebones_macro() {
        assert_eq!(
            full_macro(r#"json_typegen!("Bob", "{}");"#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::macro_default(),
            })
        );
    }

    #[test]
    fn full_macro_accepts_barebones_macro_with_options() {
        assert_eq!(
            full_macro(r#"json_typegen!("Bob", "{}", {});"#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::macro_default(),
            })
        );
    }

    #[test]
    fn full_macro_accepts_rust_whitespace_and_comments() {
        let parsed = full_macro(
            r#"
                json_typegen /* generated */ ! (
                    "Bob",
                    "{}",
                    { deny_unknown_fields }
                );
            "#,
        )
        .unwrap();

        assert_eq!(parsed.name, "Bob");
        assert!(parsed.options.deny_unknown_fields);
    }

    #[test]
    fn full_macro_rejects_other_macros_and_missing_semicolons() {
        assert!(full_macro(r#"other!("Bob", "{}");"#).is_err());
        assert!(full_macro(r#"json_typegen!("Bob", "{}")"#).is_err());
    }
}
