use crate::{EnumVariant, EnumVariantData, RecDocument, RecError, RecValue};
use std::net::SocketAddrV4;
use std::str::FromStr;
use url::Url;

pub fn validate(doc: &RecDocument) -> Result<(), RecError> {
    validate_object(&doc.root, doc)?;
    Ok(())
}

fn validate_object(obj: &crate::RecObject, doc: &RecDocument) -> Result<(), RecError> {
    for (key, value) in &obj.fields {
        validate_value(value, doc)?;
    }
    Ok(())
}

fn validate_value(value: &RecValue, doc: &RecDocument) -> Result<(), RecError> {
    match value {
        RecValue::Url(u) => validate_url(u)?,
        RecValue::Socket(s) => validate_socket(s)?,
        RecValue::Pubkey(p) => validate_pubkey(p)?,
        RecValue::Array(arr) => {
            for v in arr {
                validate_value(v, doc)?;
            }
        }
        RecValue::Object(obj) => validate_object(obj, doc)?,
        RecValue::EnumVariant {
            enum_name,
            variant,
            data,
        } => {
            if let Some(enum_def) = doc.enum_definitions.get(enum_name) {
                // Find the matching variant definition
                let variant_def = enum_def
                    .variants
                    .iter()
                    .find(|v| match v {
                        EnumVariant::Unit(name) => name == variant,
                        EnumVariant::Tuple(name, _) => name == variant,
                        EnumVariant::Struct(name, _) => name == variant,
                    })
                    .ok_or_else(|| RecError::UnknownEnumVariant {
                        enum_name: enum_name.clone(),
                        variant: variant.clone(),
                    })?;

                match (variant_def, data) {
                    (EnumVariant::Unit(_), EnumVariantData::Unit) => return Ok(()),
                    (EnumVariant::Tuple(_, expected_types), EnumVariantData::Tuple(values)) => {
                        if expected_types.len() != values.len() {
                            return Err(RecError::ValidationError(format!(
                                "Enum variant {}.{} expects {} values, got {}",
                                enum_name,
                                variant,
                                expected_types.len(),
                                values.len()
                            )));
                        }
                        for value in values {
                            validate_value(value, doc)?;
                        }
                        return Ok(());
                    }
                    (EnumVariant::Struct(_, expected_fields), EnumVariantData::Struct(fields)) => {
                        for (field_name, field_def) in expected_fields {
                            if !field_def.optional && !fields.contains_key(field_name) {
                                return Err(RecError::MissingField(format!(
                                    "{}.{}.{}",
                                    enum_name, variant, field_name
                                )));
                            }
                        }
                        for (field_name, value) in fields {
                            if !expected_fields.contains_key(field_name) {
                                return Err(RecError::ValidationError(format!(
                                    "Unknown field '{}' in {}.{}",
                                    field_name, enum_name, variant
                                )));
                            }
                            validate_value(value, doc)?;
                        }
                        return Ok(());
                    }
                    _ => {
                        return Err(RecError::ValidationError(format!(
                            "Enum variant {}.{} data type mismatch",
                            enum_name, variant
                        )));
                    }
                }
            } else {
                return Err(RecError::UnknownType(enum_name.clone()));
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_url(url: &str) -> Result<(), RecError> {
    match Url::parse(url) {
        Ok(u) => {
            if u.scheme() != "http" && u.scheme() != "https" {
                return Err(RecError::InvalidUrl(format!(
                    "URL must be HTTP or HTTPS: {}",
                    url
                )));
            }
            Ok(())
        }
        Err(_) => Err(RecError::InvalidUrl(url.to_string())),
    }
}

fn validate_socket(addr: &str) -> Result<(), RecError> {
    match SocketAddrV4::from_str(addr) {
        Ok(_) => Ok(()),
        Err(_) => Err(RecError::InvalidSocket(addr.to_string())),
    }
}

fn validate_pubkey(key: &str) -> Result<(), RecError> {
    match base58::FromBase58::from_base58(key) {
        Ok(bytes) => {
            if bytes.len() != 32 {
                Err(RecError::InvalidPubkey(format!(
                    "Invalid pubkey length: expected 32 bytes, got {}",
                    bytes.len()
                )))
            } else {
                Ok(())
            }
        }
        Err(_) => Err(RecError::InvalidPubkey(format!(
            "Invalid Base58 encoding: {}",
            key
        ))),
    }
}
