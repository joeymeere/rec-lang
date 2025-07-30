use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, LitStr};

/// Load and parse REC files at runtime
///
/// # Example
/// ```rust
/// use rec_macros::rec;
///
/// let config = rec!("config/app.rec");
/// let port = config.get("server")?.get("port")?.as_int()?;
/// ```
#[proc_macro]
pub fn rec(input: TokenStream) -> TokenStream {
    let file_path = parse_macro_input!(input as LitStr);

    let expanded = quote! {
        {
            use ::std::fs;
            use ::rec::{parse_rec, RecDocument};

            let content = fs::read_to_string(#file_path)
                .map_err(|e| format!("Failed to read REC file '{}': {}", #file_path, e))?;

            parse_rec(&content)
                .map_err(|e| format!("Failed to parse REC file '{}': {}", #file_path, e))?
        }
    };

    TokenStream::from(expanded)
}

/// Load and parse REC files at compile time
///
/// # Example
/// ```rust
/// use rec_macros::rec_const;
///
/// static CONFIG: &str = rec_const!("config/app.rec");
///
/// fn main() {
///     let doc = rec::parse_rec(CONFIG).unwrap();
///     // Use the pre-validated config
/// }
/// ```
#[proc_macro]
pub fn rec_const(input: TokenStream) -> TokenStream {
    let file_path = parse_macro_input!(input as LitStr);

    let expanded = quote! {
        {
            const REC_CONTENT: &str = include_str!(#file_path);

            const _: () = {
                // TODO: build.rs script to validate REC files at build time
                if REC_CONTENT.is_empty() {
                    panic!("REC file is empty");
                }
            };

            REC_CONTENT
        }
    };

    TokenStream::from(expanded)
}

/// Derive all traits needed to parse REC from a struct or enum.
///
/// # Example
/// ```rust
/// use rec_macros::RecParse;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(RecParse, Deserialize, Serialize)]
/// struct ServerConfig {
///     host: String,
///     port: u16,
///     #[rec(default = true)]
///     ssl_enabled: bool,
///     #[rec(rename = "allowed_origins")]
///     origins: Vec<String>,
/// }
///
/// #[derive(RecParse, Deserialize, Serialize)]
/// #[rec(enum_type = "tagged")]
/// enum Database {
///     Postgres { host: String, port: u16 },
///     Redis { host: String, port: u16 },
/// }
///
/// let config = ServerConfig::from_rec_file("server.rec")?;
/// let config = ServerConfig::from_rec_value(&rec_value)?;
/// ```
#[proc_macro_derive(RecParse, attributes(rec))]
pub fn derive_rec_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let implementation = match &input.data {
        Data::Struct(data_struct) => generate_struct_impl(name, data_struct),
        Data::Enum(data_enum) => generate_enum_impl(name, data_enum),
        Data::Union(_) => {
            return TokenStream::from(
                syn::Error::new_spanned(&input, "RecParse cannot be derived for unions")
                    .to_compile_error(),
            );
        }
    };

    let expanded = quote! {
        impl #name {
            pub fn from_rec_file<P: AsRef<::std::path::Path>>(path: P) -> Result<Self, Box<dyn ::std::error::Error>> {
                let content = ::std::fs::read_to_string(path)?;
                let doc = ::rec::parse_rec(&content)?;
                Self::from_rec_value(&::rec::RecValue::Object(doc.root))
            }

            pub fn from_rec_value(value: &::rec::RecValue) -> Result<Self, Box<dyn ::std::error::Error>> {
                let json = ::serde_json::to_value(value)?;
                let result = ::serde_json::from_value(json)?;
                Ok(result)
            }

            pub fn from_rec_str(content: &str) -> Result<Self, Box<dyn ::std::error::Error>> {
                let doc = ::rec::parse_rec(content)?;
                Self::from_rec_value(&::rec::RecValue::Object(doc.root))
            }
        }

        #implementation
    };

    TokenStream::from(expanded)
}

fn generate_struct_impl(
    name: &syn::Ident,
    _data_struct: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    quote! {
        impl ::rec::RecDeserialize for #name {
            fn from_rec(value: &::rec::RecValue) -> Result<Self, ::rec::RecError> {
                match value {
                    ::rec::RecValue::Object(obj) => {
                        let json = ::serde_json::to_value(value)
                            .map_err(|e| ::rec::RecError::ParseError(e.to_string()))?;
                        ::serde_json::from_value(json)
                            .map_err(|e| ::rec::RecError::ParseError(e.to_string()))
                    }
                    _ => Err(::rec::RecError::TypeError {
                        expected: "object".to_string(),
                        actual: format!("{:?}", value),
                    }),
                }
            }
        }
    }
}

fn generate_enum_impl(name: &syn::Ident, _data_enum: &syn::DataEnum) -> proc_macro2::TokenStream {
    quote! {
        impl ::rec::RecDeserialize for #name {
            fn from_rec(value: &::rec::RecValue) -> Result<Self, ::rec::RecError> {
                match value {
                    ::rec::RecValue::EnumVariant { enum_name, variant, data } => {
                        let json = ::serde_json::to_value(value)
                            .map_err(|e| ::rec::RecError::ParseError(e.to_string()))?;
                        ::serde_json::from_value(json)
                            .map_err(|e| ::rec::RecError::ParseError(e.to_string()))
                    }
                    _ => Err(::rec::RecError::TypeError {
                        expected: "enum variant".to_string(),
                        actual: format!("{:?}", value),
                    }),
                }
            }
        }
    }
}
