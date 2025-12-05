use core::fmt::{self, Write as _};
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use udled::{
    AsChar, AsSlice, AsStr, Buffer, EOF, Input, Next, Span, Tokenizer, TokenizerExt,
    tokenizers::WhiteSpace,
};
use udled_tokenizers::Ident;

pub trait Lookup {
    fn replace(&self, name: &str, output: &mut String) -> fmt::Result;
}

impl<K, V> Lookup for BTreeMap<K, V>
where
    K: Ord,
    K: std::borrow::Borrow<str>,
    V: fmt::Display,
{
    fn replace(&self, name: &str, output: &mut String) -> fmt::Result {
        if let Some(value) = self.get(name) {
            write!(output, "{}", value)
        } else {
            Err(fmt::Error)
        }
    }
}

impl<K, V> Lookup for HashMap<K, V>
where
    K: Hash + Eq,
    K: std::borrow::Borrow<str>,
    V: fmt::Display,
{
    fn replace(&self, name: &str, output: &mut String) -> fmt::Result {
        if let Some(value) = self.get(name) {
            write!(output, "{}", value)
        } else {
            Err(fmt::Error)
        }
    }
}

pub fn replace<T: Lookup>(input: &str, lookup: &T) -> udled::Result<String> {
    let mut input = Input::new(input);
    input.parse(Parser(lookup))
}

struct Parser<'a, T>(&'a T);

impl<'input, B, T> Tokenizer<'input, B> for Parser<'input, T>
where
    T: Lookup,
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsStr<'input> + AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    type Token = String;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, udled::Error> {
        let mut output = String::with_capacity(reader.buffer().source().as_str().len());
        let ws = WhiteSpace.many().optional();

        let mut current = 0;

        while !reader.is(EOF) {
            if reader.is("\\$") {
                reader.eat("\\$")?;
            }

            if reader.is("${") {
                if let Some(slice) =
                    Span::new(current, reader.position()).slice(reader.buffer().source().as_str())
                {
                    output.push_str(slice);
                }

                reader.eat(("${", &ws))?;

                let ident = reader.parse(Ident)?;

                if let Err(_) = self.0.replace(ident.value.as_str(), &mut output) {
                    return Err(
                        reader.error(format!("Lookup '{}' not found", ident.value.as_str()))
                    );
                }

                reader.eat((&ws, '}'))?;

                current = reader.position();
            } else {
                reader.eat(Next)?;
            }
        }

        if current != reader.position() {
            if let Some(slice) =
                Span::new(current, reader.position()).slice(reader.buffer().source().as_str())
            {
                output.push_str(slice);
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::template::{Lookup, replace};

    struct Test;

    impl Lookup for Test {
        fn replace(&self, name: &str, output: &mut String) -> core::fmt::Result {
            match name {
                "name" => {
                    output.push_str("Wilbur");
                    Ok(())
                }
                "age" => {
                    output.push_str("16");
                    Ok(())
                }
                _ => Err(core::fmt::Error),
            }
        }
    }

    #[test]
    fn test_replace() {
        assert_eq!(
            "Wilbur er 16 år gammel",
            replace("${name } er ${age} år gammel", &Test).unwrap()
        )
    }

    #[test]
    fn test_replace_btree() {
        let mut map = BTreeMap::new();
        map.insert("name", "Rasmus");

        assert_eq!("Hej, Rasmus!", replace("Hej, ${name}!", &map).unwrap())
    }
}
