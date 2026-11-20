use crate::{
    crc::crc_hex,
    tlv::{parse_all, Tlv},
    Error, Result,
};

/// Longest MPM payload accepted, in characters.
pub const MAX_PAYLOAD_CHARS: usize = 512;

/// Checks framing (size, final `6304` CRC object, checksum, PFI first) and
/// returns the top-level objects without the CRC object.
///
/// The CRC is computed over the UTF-8 bytes of everything up to and including
/// `6304`. Per-tag rules are applied by [`crate::mpm::MpmData::parse`].
pub fn validate_mpm(payload: &str) -> Result<Vec<Tlv>> {
    let char_count = payload.chars().count();
    if char_count > MAX_PAYLOAD_CHARS {
        return Err(Error::InvalidPayload(format!(
            "payload exceeds {MAX_PAYLOAD_CHARS} characters"
        )));
    }
    if payload.len() < 8 || !payload.is_char_boundary(payload.len() - 8) {
        return Err(Error::InvalidPayload(
            "missing final CRC object 6304".into(),
        ));
    }

    let (body, crc_object) = payload.split_at(payload.len() - 8);
    if !crc_object.starts_with("6304") {
        return Err(Error::InvalidPayload(
            "missing final CRC object 6304".into(),
        ));
    }

    let actual = &crc_object[4..];
    if !actual.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::InvalidPayload(
            "CRC must be four hexadecimal characters".into(),
        ));
    }

    let expected = crc_hex(format!("{body}6304").as_bytes());
    if !expected.eq_ignore_ascii_case(actual) {
        return Err(Error::CrcMismatch {
            expected,
            actual: actual.to_owned(),
        });
    }

    let objects = parse_all(body)?;
    if objects.first().map(|o| (o.tag.as_str(), o.value.as_str())) != Some(("00", "01")) {
        return Err(Error::MissingTag(
            "00 (Payload Format Indicator = 01) must be first".into(),
        ));
    }

    if objects.iter().any(|o| o.tag == "63") {
        return Err(Error::InvalidPayload(
            "CRC tag must not appear in the pre-CRC body".into(),
        ));
    }

    Ok(objects)
}
