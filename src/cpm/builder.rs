use crate::{Error, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Encodes a CPM payload that has already been constructed and validated
/// according to the CPM BER-TLV specification. Do not pass arbitrary MPM text.
pub fn encode_qr_payload(ber_tlv_payload: &[u8]) -> Result<String> {
    if ber_tlv_payload.is_empty() {
        return Err(Error::InvalidPayload("CPM payload cannot be empty".into()));
    }

    // CPM-specific construction (Payload Format Indicator CPV01, required
    // application template(s), and any applicable common template) must be
    // performed and validated before this function is called.
    Ok(STANDARD.encode(ber_tlv_payload))
}
