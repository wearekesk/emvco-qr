use crate::{
    tlv::{parse_all, Tlv},
    Result,
};

pub fn parse_mpm(payload: &str) -> Result<Vec<Tlv>> {
    parse_all(payload)
}
