use crate::{Error, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Base64-decodes the QR string. A complete validator must then parse BER-TLV,
/// validate CPV01, required application template(s), object lengths, ordering,
/// and the chosen application profile against the official CPM specification.
pub fn decode_qr_payload(encoded: &str) -> Result<Vec<u8>> {
    let decoded = STANDARD
        .decode(encoded)
        .map_err(|_| Error::InvalidPayload("invalid CPM Base64".into()))?;

    if decoded.is_empty() {
        return Err(Error::InvalidPayload("empty CPM payload".into()));
    }
    Ok(decoded)
}
