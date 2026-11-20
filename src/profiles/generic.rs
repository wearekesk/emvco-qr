use super::traits::{CpmProfile, MpmProfile, PaymentProfile};
use crate::{cpm::ber_tlv::BerTlv, tlv::Tlv, Error, Result};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GenericProfile;

impl PaymentProfile for GenericProfile {
    fn profile_id(&self) -> &'static str {
        "generic"
    }
    fn display_name(&self) -> &'static str {
        "Generic EMVCo"
    }
}

impl MpmProfile for GenericProfile {}
impl CpmProfile for GenericProfile {}

/// Minimal structural baseline only; complete this with the current EMVCo rules.
pub fn validate_generic_mpm_fields(fields: &[Tlv]) -> Result<()> {
    if fields.first().map(|f| (f.tag.as_str(), f.value.as_str())) != Some(("00", "01")) {
        return Err(Error::MissingTag(
            "00 (Payload Format Indicator = 01) must be first".into(),
        ));
    }

    let mut seen = std::collections::HashSet::new();
    for field in fields {
        if !seen.insert(field.tag.as_str()) {
            return Err(Error::DuplicateTag(field.tag.clone()));
        }
    }
    Ok(())
}

/// Placeholder only; replace with full CPM BER-TLV tree and template validation.
pub fn validate_generic_cpm_objects(objects: &[BerTlv]) -> Result<()> {
    if objects.is_empty() {
        return Err(Error::InvalidPayload("CPM object list is empty".into()));
    }
    Ok(())
}
