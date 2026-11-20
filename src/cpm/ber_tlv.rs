use crate::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BerTlv {
    pub tag: Vec<u8>,
    pub value: Vec<u8>,
}

impl BerTlv {
    pub fn new(tag: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            tag: tag.into(),
            value: value.into(),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.tag.is_empty() {
            return Err(Error::InvalidPayload("BER-TLV tag cannot be empty".into()));
        }

        let mut out = self.tag.clone();
        encode_length(self.value.len(), &mut out)?;
        out.extend_from_slice(&self.value);
        Ok(out)
    }
}

fn encode_length(length: usize, out: &mut Vec<u8>) -> Result<()> {
    if length < 0x80 {
        out.push(length as u8);
        return Ok(());
    }

    let bytes = length.to_be_bytes();
    let first_nonzero = bytes
        .iter()
        .position(|b| *b != 0)
        .unwrap_or(bytes.len() - 1);
    let encoded = &bytes[first_nonzero..];

    if encoded.len() > 4 {
        return Err(Error::InvalidPayload("BER-TLV length is too large".into()));
    }

    out.push(0x80 | encoded.len() as u8);
    out.extend_from_slice(encoded);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_length() {
        let tlv = BerTlv::new(vec![0x5A], vec![0x01, 0x02]);
        assert_eq!(tlv.encode().unwrap(), vec![0x5A, 0x02, 0x01, 0x02]);
    }

    #[test]
    fn test_long_length() {
        let val = vec![0xAA; 130];
        let tlv = BerTlv::new(vec![0x5A], val.clone());
        let encoded = tlv.encode().unwrap();
        assert_eq!(encoded[0], 0x5A);
        assert_eq!(encoded[1], 0x81);
        assert_eq!(encoded[2], 130);
        assert_eq!(&encoded[3..], &val[..]);
    }
}
