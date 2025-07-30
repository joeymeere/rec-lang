pub mod ast;
pub mod error;
pub mod parser;
pub mod validator;
pub mod value;

pub use ast::*;
pub use error::RecError;
pub use parser::parse_rec;
pub use validator::validate;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let input = r#"{
            name: "test"
            port: 8080
            enabled: true
        }"#;

        let doc = parse_rec(input).unwrap();
        assert_eq!(
            doc.root.fields.get("name").unwrap().as_string().unwrap(),
            "test"
        );
        assert_eq!(doc.root.fields.get("port").unwrap().as_int().unwrap(), 8080);
    }

    #[test]
    fn test_parse_struct_enums() {
        let input = r#"
        @enum Database {
            Postgres { host: string, port: int, ssl: bool }
            Redis { host: string, port: int }
        }
        
        {
            db: Database.Postgres {
                host: "localhost"
                port: 5432
                ssl: true
            }
        }"#;

        let doc = parse_rec(input).unwrap();
        validate(&doc).unwrap();

        match doc.root.fields.get("db").unwrap() {
            RecValue::EnumVariant {
                enum_name,
                variant,
                data,
            } => {
                assert_eq!(enum_name, "Database");
                assert_eq!(variant, "Postgres");
                match data {
                    EnumVariantData::Struct(fields) => {
                        assert_eq!(
                            fields.get("host").unwrap().as_string().unwrap(),
                            "localhost"
                        );
                        assert_eq!(fields.get("port").unwrap().as_int().unwrap(), 5432);
                        assert_eq!(fields.get("ssl").unwrap(), &RecValue::Bool(true));
                    }
                    _ => panic!("Expected struct variant"),
                }
            }
            _ => panic!("Expected enum variant"),
        }
    }

    #[test]
    fn test_serde() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        #[serde(tag = "variant", content = "data")]
        enum Database {
            Postgres { host: String, port: u16, ssl: bool },
            Redis { host: String, port: u16 },
        }

        let input = r#"
        @enum Database {
            Postgres { host: string, port: int, ssl: bool }
            Redis { host: string, port: int }
        }
        
        {
            db: Database.Postgres {
                host: "localhost"
                port: 5432
                ssl: true
            }
        }"#;

        let doc = parse_rec(input).unwrap();
        let json = serde_json::to_value(&doc.root).unwrap();

        let db_json = &json["db"];
        let db: Database = serde_json::from_value(db_json.clone()).unwrap();

        match db {
            Database::Postgres { host, port, ssl } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 5432);
                assert_eq!(ssl, true);
            }
            _ => panic!("Expected Postgres variant"),
        }
    }
}
