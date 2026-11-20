use crate::{Error, Result};

/// One MPM data object: a two-digit tag and its value.
///
/// The length written on the wire is the value's character count, as the MPM
/// specification defines it. Only the language template (tag 64) may carry
/// non-ASCII characters; [`crate::mpm::MpmData`] enforces per-tag character sets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tlv {
    pub tag: String,
    pub value: String,
}

impl Tlv {
    pub fn new(tag: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        let tag = tag.into();
        let value = value.into();

        if tag.len() != 2 || !tag.bytes().all(|b| b.is_ascii_digit()) {
            return Err(Error::InvalidTag(tag));
        }
        let length = value.chars().count();
        if length > 99 {
            return Err(Error::InvalidLength { tag, length });
        }

        Ok(Self { tag, value })
    }

    pub fn encode(&self) -> String {
        format!(
            "{}{:02}{}",
            self.tag,
            self.value.chars().count(),
            self.value
        )
    }
}

pub fn encode_all(objects: &[Tlv]) -> String {
    objects.iter().map(Tlv::encode).collect()
}

/// Parses one flat MPM TLV level. Nested templates are returned as values and
/// must be parsed recursively by the caller.
pub fn parse_all(input: &str) -> Result<Vec<Tlv>> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut result = Vec::new();

    while i < chars.len() {
        if i + 4 > chars.len() {
            return Err(Error::InvalidPayload("truncated tag/length header".into()));
        }

        let header = &chars[i..i + 4];
        if !header.iter().all(char::is_ascii_digit) {
            return Err(Error::InvalidPayload(
                "tag and length must be two ASCII digits".into(),
            ));
        }
        let tag: String = header[..2].iter().collect();
        let len = header[2..]
            .iter()
            .fold(0usize, |acc, c| acc * 10 + (*c as usize - '0' as usize));
        i += 4;

        if i + len > chars.len() {
            return Err(Error::InvalidPayload(format!(
                "tag {tag} extends past end of payload"
            )));
        }

        let value: String = chars[i..i + len].iter().collect();
        result.push(Tlv { tag, value });
        i += len;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlv_encode() {
        let tlv = Tlv::new("00", "01").unwrap();
        assert_eq!(tlv.encode(), "000201");
    }

    #[test]
    fn test_tlv_invalid_tag() {
        assert!(Tlv::new("0", "01").is_err());
        assert!(Tlv::new("0a", "01").is_err());
        assert!(Tlv::new("001", "01").is_err());
    }

    #[test]
    fn test_tlv_length_limit() {
        assert!(Tlv::new("59", "a".repeat(99)).is_ok());
        assert!(Tlv::new("59", "a".repeat(100)).is_err());
    }

    #[test]
    fn length_counts_characters_not_bytes() {
        let tlv = Tlv::new("01", "Õismäe").unwrap();
        assert_eq!(tlv.encode(), "0106Õismäe");
        assert_eq!(parse_all("0106Õismäe").unwrap(), vec![tlv]);
    }

    #[test]
    fn test_parse_all() {
        let parsed = parse_all("000201010211").unwrap();
        assert_eq!(
            parsed,
            vec![
                Tlv {
                    tag: "00".into(),
                    value: "01".into()
                },
                Tlv {
                    tag: "01".into(),
                    value: "11".into()
                }
            ]
        );
    }

    #[test]
    fn rejects_malformed_headers() {
        assert!(parse_all("0005abc").is_err());
        assert!(parse_all("000").is_err());
        assert!(parse_all("0a02ab").is_err());
    }
}
