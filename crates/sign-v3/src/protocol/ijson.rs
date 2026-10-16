//! Strict I-JSON (RFC 7493) as MSG-01 requires: valid UTF-8, no duplicate keys, no lone
//! surrogates, integers exact as doubles. It also records the paths of numbers written with a
//! fraction or exponent, so integer-only fields can be checked against the raw text.

use {
    serde_json::{Map, Number, Value},
    std::collections::HashSet,
};

#[derive(Debug, thiserror::Error)]
#[error("not I-JSON: {0}")]
pub struct IJsonError(pub String);

#[derive(Debug)]
pub struct Parsed {
    pub value: Value,
    pub non_integer: HashSet<String>,
}

impl Parsed {
    /// MSG-01: the field must be a JSON integer written without fraction or exponent.
    pub fn int(&self, path: &str) -> Result<i64, IJsonError> {
        let v = pointer(&self.value, path)
            .ok_or_else(|| IJsonError(format!("{path} missing")))?;
        if self.non_integer.contains(path) {
            return Err(IJsonError(format!("{path} must be an integer")));
        }
        v.as_i64()
            .ok_or_else(|| IJsonError(format!("{path} must be an integer")))
    }

    pub fn opt_int(&self, path: &str) -> Result<Option<i64>, IJsonError> {
        match pointer(&self.value, path) {
            None | Some(Value::Null) => Ok(None),
            Some(_) => self.int(path).map(Some),
        }
    }
}

fn pointer<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() { Some(v) } else { v.pointer(path) }
}

/// The crate's single JSON entry point: everything parsed goes through this parser.
pub fn value(bytes: &[u8]) -> Option<Value> {
    parse_bytes(bytes).ok().map(|p| p.value)
}

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    serde_json::from_value(value(bytes)?).ok()
}

pub fn from_value<T: serde::de::DeserializeOwned>(v: &Value) -> Option<T> {
    serde_json::from_value(v.clone()).ok()
}

pub fn parse_bytes(bytes: &[u8]) -> Result<Parsed, IJsonError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| IJsonError("invalid UTF-8".into()))?;
    parse(text)
}

pub fn parse(text: &str) -> Result<Parsed, IJsonError> {
    let mut p =
        Parser { s: text.as_bytes(), i: 0, non_integer: HashSet::new() };
    let value = p.value("")?;
    p.ws();
    if p.i != p.s.len() {
        return Err(p.err("trailing data"));
    }
    Ok(Parsed { value, non_integer: p.non_integer })
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    non_integer: HashSet<String>,
}

impl Parser<'_> {
    fn err(&self, m: &str) -> IJsonError {
        IJsonError(format!("{m} at {}", self.i))
    }

    fn ws(&mut self) {
        while self.i < self.s.len()
            && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r')
        {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn lit(&mut self, word: &str, v: Value) -> Result<Value, IJsonError> {
        if self.s[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(self.err("unexpected token"))
        }
    }

    fn value(&mut self, path: &str) -> Result<Value, IJsonError> {
        self.ws();
        match self.peek() {
            Some(b'{') => self.object(path),
            Some(b'[') => self.array(path),
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b't') => self.lit("true", Value::Bool(true)),
            Some(b'f') => self.lit("false", Value::Bool(false)),
            Some(b'n') => self.lit("null", Value::Null),
            Some(_) => self.number(path),
            None => Err(self.err("unexpected end")),
        }
    }

    fn object(&mut self, path: &str) -> Result<Value, IJsonError> {
        self.i += 1;
        let mut map = Map::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Value::Object(map));
        }
        loop {
            self.ws();
            if self.peek() != Some(b'"') {
                return Err(self.err("expected key"));
            }
            let key = self.string()?;
            if map.contains_key(&key) {
                return Err(self.err(&format!("duplicate key \"{key}\"")));
            }
            self.ws();
            if self.peek() != Some(b':') {
                return Err(self.err("expected colon"));
            }
            self.i += 1;
            let child =
                format!("{path}/{}", key.replace('~', "~0").replace('/', "~1"));
            let v = self.value(&child)?;
            map.insert(key, v);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Value::Object(map));
                }
                _ => return Err(self.err("expected , or }")),
            }
        }
    }

    fn array(&mut self, path: &str) -> Result<Value, IJsonError> {
        self.i += 1;
        let mut out = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Value::Array(out));
        }
        loop {
            let v = self.value(&format!("{path}/{}", out.len()))?;
            out.push(v);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Value::Array(out));
                }
                _ => return Err(self.err("expected , or ]")),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, IJsonError> {
        let h = self
            .s
            .get(self.i..self.i + 4)
            .ok_or_else(|| self.err("bad unicode escape"))?;
        let h = std::str::from_utf8(h)
            .map_err(|_| self.err("bad unicode escape"))?;
        let v = u32::from_str_radix(h, 16)
            .map_err(|_| self.err("bad unicode escape"))?;
        self.i += 4;
        Ok(v)
    }

    fn string(&mut self) -> Result<String, IJsonError> {
        self.i += 1;
        let mut out = String::new();
        loop {
            let c =
                self.peek().ok_or_else(|| self.err("unterminated string"))?;
            match c {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    let e = *self
                        .s
                        .get(self.i + 1)
                        .ok_or_else(|| self.err("bad escape"))?;
                    self.i += 2;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hi = self.hex4()?;
                            let cp = if (0xD800..0xDC00).contains(&hi) {
                                if self.s.get(self.i..self.i + 2)
                                    != Some(b"\\u")
                                {
                                    return Err(self.err("lone surrogate"));
                                }
                                self.i += 2;
                                let lo = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return Err(self.err("lone surrogate"));
                                }
                                0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                            } else if (0xDC00..0xE000).contains(&hi) {
                                return Err(self.err("lone surrogate"));
                            } else {
                                hi
                            };
                            out.push(
                                char::from_u32(cp).ok_or_else(|| {
                                    self.err("bad code point")
                                })?,
                            );
                        }
                        _ => return Err(self.err("bad escape")),
                    }
                }
                c if c < 0x20 => {
                    return Err(self.err("control character in string"));
                }
                _ => {
                    // Copy one UTF-8 character; the input was validated as UTF-8.
                    let rest = std::str::from_utf8(&self.s[self.i..])
                        .map_err(|_| self.err("invalid UTF-8"))?;
                    let ch = rest
                        .chars()
                        .next()
                        .ok_or_else(|| self.err("unterminated string"))?;
                    out.push(ch);
                    self.i += ch.len_utf8();
                }
            }
        }
    }

    fn number(&mut self, path: &str) -> Result<Value, IJsonError> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        match self.peek() {
            Some(b'0') => self.i += 1,
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.i += 1;
                }
            }
            _ => return Err(self.err("bad number")),
        }
        let mut integer = true;
        if self.peek() == Some(b'.') {
            integer = false;
            self.i += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.err("bad number"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.i += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            integer = false;
            self.i += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.i += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.err("bad number"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.i += 1;
            }
        }
        let lit = std::str::from_utf8(&self.s[start..self.i]).expect("ascii");
        if integer {
            let v: i128 = lit
                .parse()
                .map_err(|_| self.err("integer not exact as a double"))?;
            if (v as f64) as i128 != v || v.unsigned_abs() > (1u128 << 63) {
                return Err(self.err("integer not exact as a double"));
            }
            return Ok(Value::Number(Number::from(v as i64)));
        }
        self.non_integer.insert(path.to_string());
        let f: f64 = lit.parse().map_err(|_| self.err("bad number"))?;
        Number::from_f64(f)
            .map(Value::Number)
            .ok_or_else(|| self.err("number out of range"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_what_ijson_forbids() {
        assert!(parse(r#"{"a":1,"a":2}"#).unwrap_err().0.contains("duplicate"));
        assert!(
            parse(r#"{"a":"\ud800"}"#).unwrap_err().0.contains("surrogate")
        );
        assert!(
            parse(r#"{"a":"\udc00"}"#).unwrap_err().0.contains("surrogate")
        );
        assert!(
            parse(r#"{"a":9007199254740993}"#).unwrap_err().0.contains("exact")
        );
        assert!(parse(r#"{"a":1} x"#).unwrap_err().0.contains("trailing"));
        assert!(
            parse_bytes(&[b'"', 0xff, b'"']).unwrap_err().0.contains("UTF-8")
        );
        assert!(parse("{\"a\":\"\u{1}\"}").is_err());
    }

    #[test]
    fn accepts_valid_and_tracks_non_integers() {
        let p = parse(r#"{"a":1.5,"b":[1e3,2],"c":"😀","d":-0}"#).unwrap();
        assert_eq!(
            p.non_integer,
            HashSet::from(["/a".to_string(), "/b/0".to_string()])
        );
        assert_eq!(p.value["c"], "😀");
        assert_eq!(p.int("/b/1").unwrap(), 2);
        assert!(p.int("/a").is_err());
        assert!(parse(r#"{"a":9007199254740992}"#).is_ok());
    }
}
