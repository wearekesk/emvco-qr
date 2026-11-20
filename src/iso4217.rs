//! ISO 4217 currency codes as MPM tag 53 needs them (numeric), with the minor
//! unit count that bounds the decimals allowed in tag 54.

use crate::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Currency {
    pub alpha: &'static str,
    pub numeric: &'static str,
    pub minor_units: u8,
}

const fn c(alpha: &'static str, numeric: &'static str, minor_units: u8) -> Currency {
    Currency {
        alpha,
        numeric,
        minor_units,
    }
}

/// Sorted by alpha code.
pub const CURRENCIES: &[Currency] = &[
    c("AED", "784", 2),
    c("AFN", "971", 2),
    c("ALL", "008", 2),
    c("AMD", "051", 2),
    c("ANG", "532", 2),
    c("AOA", "973", 2),
    c("ARS", "032", 2),
    c("AUD", "036", 2),
    c("AWG", "533", 2),
    c("AZN", "944", 2),
    c("BAM", "977", 2),
    c("BBD", "052", 2),
    c("BDT", "050", 2),
    c("BGN", "975", 2),
    c("BHD", "048", 3),
    c("BIF", "108", 0),
    c("BMD", "060", 2),
    c("BND", "096", 2),
    c("BOB", "068", 2),
    c("BRL", "986", 2),
    c("BSD", "044", 2),
    c("BTN", "064", 2),
    c("BWP", "072", 2),
    c("BYN", "933", 2),
    c("BZD", "084", 2),
    c("CAD", "124", 2),
    c("CDF", "976", 2),
    c("CHF", "756", 2),
    c("CLP", "152", 0),
    c("CNY", "156", 2),
    c("COP", "170", 2),
    c("CRC", "188", 2),
    c("CUP", "192", 2),
    c("CVE", "132", 2),
    c("CZK", "203", 2),
    c("DJF", "262", 0),
    c("DKK", "208", 2),
    c("DOP", "214", 2),
    c("DZD", "012", 2),
    c("EGP", "818", 2),
    c("ERN", "232", 2),
    c("ETB", "230", 2),
    c("EUR", "978", 2),
    c("FJD", "242", 2),
    c("FKP", "238", 2),
    c("GBP", "826", 2),
    c("GEL", "981", 2),
    c("GHS", "936", 2),
    c("GIP", "292", 2),
    c("GMD", "270", 2),
    c("GNF", "324", 0),
    c("GTQ", "320", 2),
    c("GYD", "328", 2),
    c("HKD", "344", 2),
    c("HNL", "340", 2),
    c("HTG", "332", 2),
    c("HUF", "348", 2),
    c("IDR", "360", 2),
    c("ILS", "376", 2),
    c("INR", "356", 2),
    c("IQD", "368", 3),
    c("IRR", "364", 2),
    c("ISK", "352", 0),
    c("JMD", "388", 2),
    c("JOD", "400", 3),
    c("JPY", "392", 0),
    c("KES", "404", 2),
    c("KGS", "417", 2),
    c("KHR", "116", 2),
    c("KMF", "174", 0),
    c("KRW", "410", 0),
    c("KWD", "414", 3),
    c("KYD", "136", 2),
    c("KZT", "398", 2),
    c("LAK", "418", 2),
    c("LBP", "422", 2),
    c("LKR", "144", 2),
    c("LRD", "430", 2),
    c("LSL", "426", 2),
    c("LYD", "434", 3),
    c("MAD", "504", 2),
    c("MDL", "498", 2),
    c("MGA", "969", 2),
    c("MKD", "807", 2),
    c("MMK", "104", 2),
    c("MNT", "496", 2),
    c("MOP", "446", 2),
    c("MRU", "929", 2),
    c("MUR", "480", 2),
    c("MVR", "462", 2),
    c("MWK", "454", 2),
    c("MXN", "484", 2),
    c("MYR", "458", 2),
    c("MZN", "943", 2),
    c("NAD", "516", 2),
    c("NGN", "566", 2),
    c("NIO", "558", 2),
    c("NOK", "578", 2),
    c("NPR", "524", 2),
    c("NZD", "554", 2),
    c("OMR", "512", 3),
    c("PAB", "590", 2),
    c("PEN", "604", 2),
    c("PGK", "598", 2),
    c("PHP", "608", 2),
    c("PKR", "586", 2),
    c("PLN", "985", 2),
    c("PYG", "600", 0),
    c("QAR", "634", 2),
    c("RON", "946", 2),
    c("RSD", "941", 2),
    c("RUB", "643", 2),
    c("RWF", "646", 0),
    c("SAR", "682", 2),
    c("SBD", "090", 2),
    c("SCR", "690", 2),
    c("SDG", "938", 2),
    c("SEK", "752", 2),
    c("SGD", "702", 2),
    c("SHP", "654", 2),
    c("SLE", "925", 2),
    c("SOS", "706", 2),
    c("SRD", "968", 2),
    c("SSP", "728", 2),
    c("STN", "930", 2),
    c("SVC", "222", 2),
    c("SYP", "760", 2),
    c("SZL", "748", 2),
    c("THB", "764", 2),
    c("TJS", "972", 2),
    c("TMT", "934", 2),
    c("TND", "788", 3),
    c("TOP", "776", 2),
    c("TRY", "949", 2),
    c("TTD", "780", 2),
    c("TWD", "901", 2),
    c("TZS", "834", 2),
    c("UAH", "980", 2),
    c("UGX", "800", 0),
    c("USD", "840", 2),
    c("UYU", "858", 2),
    c("UZS", "860", 2),
    c("VES", "928", 2),
    c("VND", "704", 0),
    c("VUV", "548", 0),
    c("WST", "882", 2),
    c("XAF", "950", 0),
    c("XCD", "951", 2),
    c("XOF", "952", 0),
    c("XPF", "953", 0),
    c("YER", "886", 2),
    c("ZAR", "710", 2),
    c("ZMW", "967", 2),
    c("ZWG", "924", 2),
];

pub fn by_alpha(alpha: &str) -> Option<&'static Currency> {
    let alpha = alpha.to_ascii_uppercase();
    CURRENCIES
        .binary_search_by(|c| c.alpha.cmp(alpha.as_str()))
        .ok()
        .map(|i| &CURRENCIES[i])
}

pub fn by_numeric(numeric: &str) -> Option<&'static Currency> {
    CURRENCIES.iter().find(|c| c.numeric == numeric)
}

/// Formats an amount held in minor units as a tag 54 value: no separators,
/// a `.` only when the currency has minor units, trailing zeros trimmed.
pub fn format_amount(minor: u64, currency: &Currency) -> Result<String> {
    let scale = 10u64.pow(u32::from(currency.minor_units));
    let whole = minor / scale;
    let frac = minor % scale;
    let value = if frac == 0 {
        whole.to_string()
    } else {
        let digits = format!("{frac:0width$}", width = currency.minor_units as usize);
        format!("{whole}.{}", digits.trim_end_matches('0'))
    };
    if value.len() > 13 {
        return Err(Error::InvalidLength {
            tag: "54".into(),
            length: value.len(),
        });
    }
    Ok(value)
}

/// Validates a tag 54 value: up to 13 characters, digits with at most one
/// `.`, no more decimals than `currency` allows, and greater than zero.
pub fn validate_amount(value: &str, currency: Option<&Currency>) -> Result<()> {
    let invalid = || Error::InvalidPayload(format!("invalid transaction amount {value:?}"));
    if value.is_empty() || value.len() > 13 {
        return Err(invalid());
    }
    let (whole, frac) = match value.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (value, None),
    };
    let digits_only = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if !digits_only(whole) || frac.is_some_and(|f| !digits_only(f)) {
        return Err(invalid());
    }
    if whole.is_empty() && frac.is_none_or(str::is_empty) {
        return Err(invalid());
    }
    if let (Some(f), Some(cur)) = (frac, currency) {
        if f.len() > usize::from(cur.minor_units) {
            return Err(invalid());
        }
    }
    if !value.bytes().any(|b| (b'1'..=b'9').contains(&b)) {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_and_unique() {
        assert!(CURRENCIES.windows(2).all(|w| w[0].alpha < w[1].alpha));
        let mut numerics: Vec<_> = CURRENCIES.iter().map(|c| c.numeric).collect();
        numerics.sort_unstable();
        numerics.dedup();
        assert_eq!(numerics.len(), CURRENCIES.len());
        assert!(CURRENCIES
            .iter()
            .all(|c| c.numeric.len() == 3 && c.numeric.bytes().all(|b| b.is_ascii_digit())));
    }

    #[test]
    fn lookups() {
        assert_eq!(by_alpha("eur").unwrap().numeric, "978");
        assert_eq!(by_numeric("356").unwrap().alpha, "INR");
        assert!(by_alpha("XXX").is_none());
    }

    #[test]
    fn formats_minor_units() {
        let eur = by_alpha("EUR").unwrap();
        assert_eq!(format_amount(2550, eur).unwrap(), "25.5");
        assert_eq!(format_amount(2500, eur).unwrap(), "25");
        assert_eq!(format_amount(1, eur).unwrap(), "0.01");
        let jpy = by_alpha("JPY").unwrap();
        assert_eq!(format_amount(1200, jpy).unwrap(), "1200");
        let kwd = by_alpha("KWD").unwrap();
        assert_eq!(format_amount(1005, kwd).unwrap(), "1.005");
    }

    #[test]
    fn validates_amounts() {
        let eur = by_alpha("EUR");
        let jpy = by_alpha("JPY");
        assert!(validate_amount("25.50", eur).is_ok());
        assert!(validate_amount("25.505", eur).is_err());
        assert!(validate_amount("25.5", jpy).is_err());
        assert!(validate_amount("1,000", eur).is_err());
        assert!(validate_amount("0", eur).is_err());
        assert!(validate_amount("0.00", eur).is_err());
        assert!(validate_amount(".", eur).is_err());
        assert!(validate_amount("-1", eur).is_err());
        assert!(validate_amount("12345678901234", None).is_err());
    }
}
