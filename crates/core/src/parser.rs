use crate::{
    EnumDef, EnumVariant, EnumVariantData, FieldDef, RecDocument, RecError, RecObject, RecType,
    RecValue, TypeDef,
};
use indexmap::IndexMap;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair},
};
use std::collections::HashMap;

pub fn parse_rec(input: &str) -> Result<RecDocument, RecError> {
    match document(input) {
        Ok((_, doc)) => Ok(doc),
        Err(e) => Err(RecError::ParseError(format!("{:?}", e))),
    }
}

fn document(input: &str) -> IResult<&str, RecDocument> {
    let (input, includes) = many0(include_statement).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, enums) = many0(ws(enum_definition)).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, types) = many0(ws(type_definition)).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, root) = object.parse(input)?;

    let mut enum_map = HashMap::new();
    for e in enums {
        enum_map.insert(e.name.clone(), e);
    }

    let mut type_map = HashMap::new();
    for t in types {
        type_map.insert(t.name.clone(), t);
    }

    Ok((
        input,
        RecDocument {
            includes,
            type_definitions: type_map,
            enum_definitions: enum_map,
            root,
        },
    ))
}

fn include_statement(input: &str) -> IResult<&str, String> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("#include")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, path) = string_literal(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, path))
}

fn enum_definition(input: &str) -> IResult<&str, EnumDef> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("@enum")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, variants) = separated_list0(ws(char(',')), enum_variant_def).parse(input)?;
    let (input, _) = opt(char(',')).parse(input)?;
    let (input, _) = ws(char('}')).parse(input)?;

    Ok((
        input,
        EnumDef {
            name: name.to_string(),
            variants,
        },
    ))
}

fn enum_variant_def(input: &str) -> IResult<&str, EnumVariant> {
    let (input, _) = multispace0(input)?;
    let (input, name) = identifier(input)?;

    if let Ok((input2, _)) = char::<&str, nom::error::Error<&str>>('{')(input) {
        let (input2, fields) = many0(field_definition).parse(input2)?;
        let (input2, _) = ws(char('}')).parse(input2)?;
        let mut field_map = IndexMap::new();
        for (fname, fdef) in fields {
            field_map.insert(fname, fdef);
        }
        return Ok((input2, EnumVariant::Struct(name.to_string(), field_map)));
    }

    if let Ok((input2, _)) = char::<&str, nom::error::Error<&str>>('(')(input) {
        let (input2, types) = separated_list0(ws(char(',')), type_expr).parse(input2)?;
        let (input2, _) = char(')')(input2)?;
        return Ok((input2, EnumVariant::Tuple(name.to_string(), types)));
    }

    Ok((input, EnumVariant::Unit(name.to_string())))
}

fn type_definition(input: &str) -> IResult<&str, TypeDef> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("@type")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, fields) = many0(field_definition).parse(input)?;
    let (input, _) = ws(char('}')).parse(input)?;

    let mut field_map = IndexMap::new();
    for (fname, fdef) in fields {
        field_map.insert(fname, fdef);
    }

    Ok((
        input,
        TypeDef {
            name: name.to_string(),
            fields: field_map,
        },
    ))
}

fn field_definition(input: &str) -> IResult<&str, (String, FieldDef)> {
    let (input, _) = multispace0(input)?;
    let (input, name) = identifier(input)?;
    let (input, optional) = opt(char('?')).parse(input)?;
    let (input, _) = ws(char(':')).parse(input)?;
    let (input, ty) = type_expr(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        (
            name.to_string(),
            FieldDef {
                ty,
                optional: optional.is_some(),
            },
        ),
    ))
}

fn type_expr(input: &str) -> IResult<&str, RecType> {
    alt((
        map(tag("string"), |_| RecType::String),
        map(tag("int"), |_| RecType::Int),
        map(tag("float"), |_| RecType::Float),
        map(tag("bool"), |_| RecType::Bool),
        map(tag("url"), |_| RecType::Url),
        map(tag("socket"), |_| RecType::Socket),
        map(tag("pubkey"), |_| RecType::Pubkey),
        array_type,
        map(identifier, |s| RecType::Object(s.to_string())),
    ))
    .parse(input)
}

fn array_type(input: &str) -> IResult<&str, RecType> {
    let (input, _) = char('[')(input)?;
    let (input, inner) = type_expr(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, RecType::Array(Box::new(inner))))
}

fn object(input: &str) -> IResult<&str, RecObject> {
    let (input, _) = ws(char('{')).parse(input)?;
    let (input, pairs) = separated_list0(ws(char(',')), key_value_pair).parse(input)?;
    let (input, _) = opt(char(',')).parse(input)?;
    let (input, _) = ws(char('}')).parse(input)?;

    let mut fields = IndexMap::new();
    for (k, v) in pairs {
        fields.insert(k, v);
    }

    Ok((input, RecObject { fields }))
}

fn key_value_pair(input: &str) -> IResult<&str, (String, RecValue)> {
    let (input, key) = ws(identifier).parse(input)?;
    let (input, _) = ws(char(':')).parse(input)?;
    let (input, value) = ws(rec_value).parse(input)?;
    Ok((input, (key.to_string(), value)))
}

fn rec_value(input: &str) -> IResult<&str, RecValue> {
    alt((
        map(string_literal, RecValue::String),
        map(float, RecValue::Float),
        map(integer, RecValue::Int),
        map(boolean, RecValue::Bool),
        map(tag("null"), |_| RecValue::Null),
        url_value,
        socket_value,
        pubkey_value,
        enum_variant,
        map(array, RecValue::Array),
        typed_object,
        map(object, RecValue::Object),
    ))
    .parse(input)
}

fn typed_object(input: &str) -> IResult<&str, RecValue> {
    let (input, _type_name) = identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, obj) = object(input)?;
    Ok((input, RecValue::Object(obj)))
}

fn url_value(input: &str) -> IResult<&str, RecValue> {
    let (input, _) = tag("url(")(input)?;
    let (input, url) = string_literal(input)?;
    let (input, _) = char(')')(input)?;
    Ok((input, RecValue::Url(url)))
}

fn socket_value(input: &str) -> IResult<&str, RecValue> {
    let (input, _) = tag("socket(")(input)?;
    let (input, addr) = string_literal(input)?;
    let (input, _) = char(')')(input)?;
    Ok((input, RecValue::Socket(addr)))
}

fn pubkey_value(input: &str) -> IResult<&str, RecValue> {
    let (input, _) = tag("pubkey(")(input)?;
    let (input, key) = string_literal(input)?;
    let (input, _) = char(')')(input)?;
    Ok((input, RecValue::Pubkey(key)))
}

fn enum_variant(input: &str) -> IResult<&str, RecValue> {
    let (input, enum_name) = identifier(input)?;
    let (input, _) = char('.')(input)?;
    let (input, variant) = identifier(input)?;

    if let Ok((input2, _)) = multispace0::<&str, nom::error::Error<&str>>(input) {
        if let Ok((input2, _)) = char::<&str, nom::error::Error<&str>>('{')(input2) {
            let (input2, pairs) = separated_list0(ws(char(',')), key_value_pair).parse(input2)?;
            let (input2, _) = opt(char(',')).parse(input2)?;
            let (input2, _) = ws(char('}')).parse(input2)?;

            let mut fields = IndexMap::new();
            for (k, v) in pairs {
                fields.insert(k, v);
            }

            return Ok((
                input2,
                RecValue::EnumVariant {
                    enum_name: enum_name.to_string(),
                    variant: variant.to_string(),
                    data: EnumVariantData::Struct(fields),
                },
            ));
        }
    }

    if let Ok((input2, _)) = char::<&str, nom::error::Error<&str>>('(')(input) {
        let (input2, values) = separated_list0(ws(char(',')), ws(rec_value)).parse(input2)?;
        let (input2, _) = char(')').parse(input2)?;

        return Ok((
            input2,
            RecValue::EnumVariant {
                enum_name: enum_name.to_string(),
                variant: variant.to_string(),
                data: EnumVariantData::Tuple(values),
            },
        ));
    }

    Ok((
        input,
        RecValue::EnumVariant {
            enum_name: enum_name.to_string(),
            variant: variant.to_string(),
            data: EnumVariantData::Unit,
        },
    ))
}

fn array(input: &str) -> IResult<&str, Vec<RecValue>> {
    let (input, _) = char('[')(input)?;
    let (input, values) = separated_list0(ws(char(',')), ws(rec_value)).parse(input)?;
    let (input, _) = opt(char(',')).parse(input)?;
    let (input, _) = ws(char(']')).parse(input)?;
    Ok((input, values))
}

fn string_literal(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"')(input)?;
    let (input, content) = take_while(|c| c != '"')(input)?;
    let (input, _) = char('"')(input)?;
    Ok((input, content.to_string()))
}

fn integer(input: &str) -> IResult<&str, i64> {
    let (input, sign) = opt(char('-')).parse(input)?;
    let (input, digits) = digit1(input)?;
    let value = digits.parse::<i64>().unwrap();
    Ok((input, if sign.is_some() { -value } else { value }))
}

fn float(input: &str) -> IResult<&str, f64> {
    let (input, sign) = opt(char('-')).parse(input)?;
    let (input, whole) = digit1(input)?;
    let (input, _) = char('.')(input)?;
    let (input, decimal) = digit1(input)?;
    let value = format!("{}.{}", whole, decimal).parse::<f64>().unwrap();
    Ok((input, if sign.is_some() { -value } else { value }))
}

fn boolean(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("true")), value(false, tag("false")))).parse(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))
    .parse(input)
}

fn ws<'a, F, O>(inner: F) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
where
    F: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
{
    delimited(multispace0, inner, multispace0)
}
