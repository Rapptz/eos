#![no_std]
extern crate alloc;

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    str::FromStr,
    string::{String, ToString},
    vec::Vec,
};

use proc_macro::TokenStream;

fn emit_error(text: Cow<'static, str>) -> TokenStream {
    // Create a `compile_error!("s")` invocation
    let invoke = format!("compile_error!({:?})", text);
    TokenStream::from_str(invoke.as_str()).unwrap()
}

struct CodeIterator<'a> {
    data: &'a [u8],
    inside_directive: bool,
}

impl<'a> CodeIterator<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            data: s.as_bytes(),
            inside_directive: s.as_bytes().first() == Some(&b'%'),
        }
    }
}

fn parse_directive(directive: u8) -> Result<&'static str, String> {
    match directive {
        b'a' => Ok("eos::fmt::FormatSpecKind::AbbreviatedWeekday"),
        b'A' => Ok("eos::fmt::FormatSpecKind::FullWeekday"),
        b'w' => Ok("eos::fmt::FormatSpecKind::Weekday"),
        b'u' => Ok("eos::fmt::FormatSpecKind::IsoWeekday"),
        b'd' => Ok("eos::fmt::FormatSpecKind::Day"),
        b'j' => Ok("eos::fmt::FormatSpecKind::Ordinal"),
        b'b' => Ok("eos::fmt::FormatSpecKind::AbbreviatedMonth"),
        b'B' => Ok("eos::fmt::FormatSpecKind::FullMonth"),
        b'm' => Ok("eos::fmt::FormatSpecKind::Month"),
        b'Y' => Ok("eos::fmt::FormatSpecKind::Year"),
        b'y' => Ok("eos::fmt::FormatSpecKind::SignedYear"),
        b'G' => Ok("eos::fmt::FormatSpecKind::IsoWeekYear"),
        b'V' => Ok("eos::fmt::FormatSpecKind::IsoWeek"),
        b'H' => Ok("eos::fmt::FormatSpecKind::Hour"),
        b'I' => Ok("eos::fmt::FormatSpecKind::Hour12"),
        b'p' => Ok("eos::fmt::FormatSpecKind::Meridian"),
        b'M' => Ok("eos::fmt::FormatSpecKind::Minute"),
        b'S' => Ok("eos::fmt::FormatSpecKind::Second"),
        b'f' => Ok("eos::fmt::FormatSpecKind::Nanosecond"),
        b'o' => Ok("eos::fmt::FormatSpecKind::UtcOffset"),
        b'z' => Ok("eos::fmt::FormatSpecKind::UtcOffsetBrief"),
        b'Z' => Ok("eos::fmt::FormatSpecKind::ZoneName"),
        b'%' => Ok("eos::fmt::FormatSpecKind::Escape"),
        b'_' | b'#' => Err("expected specifier after `_` or `#` modifier".to_owned()),
        _ => Err(format!("unexpected specifier (`{}`)", directive as char)),
    }
}

impl<'a> Iterator for CodeIterator<'a> {
    type Item = Result<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inside_directive {
            self.inside_directive = false;
            match self.data {
                [b'%', b'#', directive, rest @ ..] => {
                    self.data = rest;
                    Some(
                        parse_directive(*directive)
                            .map(|s| format!("eos::fmt::FormatSpec::new({}).with_no_padding()", s)),
                    )
                }
                [b'%', b'_', directive, rest @ ..] => {
                    self.data = rest;
                    Some(
                        parse_directive(*directive)
                            .map(|s| format!("eos::fmt::FormatSpec::new({}).with_space_padding()", s)),
                    )
                }
                [b'%', directive, rest @ ..] => {
                    self.data = rest;
                    Some(parse_directive(*directive).map(|s| format!("eos::fmt::FormatSpec::new({})", s)))
                }
                [b'%'] => Some(Err("expected specifier after `%`".to_owned())),
                _ => Some(Err("expected specifier".to_owned())),
            }
        } else if self.data.is_empty() {
            None
        } else {
            match self.data.iter().position(|&c| c == b'%') {
                None => {
                    let raw = core::str::from_utf8(self.data).expect("not valid UTF-8");
                    self.data = &self.data[self.data.len()..];
                    Some(Ok(format!("eos::fmt::FormatSpec::raw({:?})", raw)))
                }
                Some(idx) => {
                    let (raw, rest) = self.data.split_at(idx);
                    self.inside_directive = true;
                    self.data = rest;
                    let raw = core::str::from_utf8(raw).expect("not valid UTF-8");
                    Some(Ok(format!("eos::fmt::FormatSpec::raw({:?})", raw)))
                }
            }
        }
    }
}

#[proc_macro]
pub fn format_spec(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();
    let tree = match tokens.next() {
        Some(tree) => tree,
        None => return emit_error("missing string literal".into()),
    };

    if tokens.next().is_some() {
        return emit_error("too many arguments in macro call".into());
    }

    let lit = tree.to_string();
    let inner = match lit.as_bytes() {
        [b'"', .., b'"'] => &lit[1..lit.len() - 1],
        _ => return emit_error(format!("expected string literal, received `{}`", lit).into()),
    };

    let fragments: Result<Vec<_>, _> = CodeIterator::new(inner).collect();
    match fragments {
        Ok(code) => {
            // Convert each element into an array...
            let code = format!("[{}]", code.join(",\n"));
            TokenStream::from_str(&code).expect("output code did not compile")
        }
        Err(err) => emit_error(err.into()),
    }
}
