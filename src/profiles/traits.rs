use crate::{cpm::ber_tlv::BerTlv, tlv::Tlv, Result};

pub trait PaymentProfile: Send + Sync {
    fn profile_id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
}

/// MPM-specific rules. Default methods provide generic baseline behavior.
pub trait MpmProfile: PaymentProfile {
    fn validate_fields(&self, fields: &[Tlv]) -> Result<()> {
        crate::profiles::generic::validate_generic_mpm_fields(fields)
    }

    /// Empty by default: generic EMVCo must not invent scheme identifiers.
    fn merchant_account_info(&self) -> Result<Vec<Tlv>> {
        Ok(Vec::new())
    }
}

/// CPM has a separate BER-TLV model and separate validation rules.
pub trait CpmProfile: PaymentProfile {
    fn validate_objects(&self, objects: &[BerTlv]) -> Result<()> {
        crate::profiles::generic::validate_generic_cpm_objects(objects)
    }
}
