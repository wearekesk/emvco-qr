use std::collections::HashSet;

use super::{validate::MAX_PAYLOAD_CHARS, validate_mpm, MpmPayload};
use crate::{
    crc::crc_hex,
    iso4217,
    text::is_printable_ascii,
    tlv::{encode_all, parse_all, Tlv},
    Error, Result,
};

/// Tag 01: whether the same QR is reused (static) or made for one payment (dynamic).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointOfInitiation {
    Static,
    Dynamic,
}

impl PointOfInitiation {
    pub fn code(self) -> &'static str {
        match self {
            Self::Static => "11",
            Self::Dynamic => "12",
        }
    }

    fn from_code(code: &str) -> Result<Self> {
        match code {
            "11" => Ok(Self::Static),
            "12" => Ok(Self::Dynamic),
            other => Err(Error::InvalidPayload(format!(
                "invalid point of initiation method {other:?}"
            ))),
        }
    }
}

/// Merchant account information, tags 02–51.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MerchantAccount {
    /// Tags 02–25: a primitive value reserved for card networks.
    Primitive { tag: String, value: String },
    /// Tags 26–51: a template whose sub-tag 00 is the globally unique
    /// identifier of the scheme that defines the remaining sub-tags.
    Template {
        tag: String,
        gui: String,
        fields: Vec<Tlv>,
    },
}

impl MerchantAccount {
    pub fn tag(&self) -> &str {
        match self {
            Self::Primitive { tag, .. } | Self::Template { tag, .. } => tag,
        }
    }

    /// The sub-tag value of a template, or `None` for a primitive entry.
    pub fn field(&self, sub_tag: &str) -> Option<&str> {
        match self {
            Self::Template { fields, .. } => fields
                .iter()
                .find(|f| f.tag == sub_tag)
                .map(|f| f.value.as_str()),
            Self::Primitive { .. } => None,
        }
    }

    fn parse(tag: String, value: String) -> Result<Self> {
        let n = tag_number(&tag);
        if (2..=25).contains(&n) {
            return Ok(Self::Primitive { tag, value });
        }
        let mut children = parse_all(&value)?;
        ensure_unique(&children, &format!("merchant account {tag}"))?;
        let gui_index = children
            .iter()
            .position(|c| c.tag == "00")
            .ok_or_else(|| Error::MissingTag(format!("{tag}/00 (globally unique identifier)")))?;
        let gui = children.remove(gui_index).value;
        Ok(Self::Template {
            tag,
            gui,
            fields: children,
        })
    }

    fn validate(&self) -> Result<()> {
        let n = tag_number(self.tag());
        if !(2..=51).contains(&n) {
            return Err(Error::InvalidTag(self.tag().to_owned()));
        }
        match self {
            Self::Primitive { tag, value } => {
                if (26..=51).contains(&n) {
                    return Err(Error::InvalidPayload(format!(
                        "merchant account {tag} must be a template"
                    )));
                }
                ascii_field(tag, value, 1, 99)
            }
            Self::Template { tag, gui, fields } => {
                if n <= 25 {
                    return Err(Error::InvalidPayload(format!(
                        "merchant account {tag} must be primitive"
                    )));
                }
                ascii_field(&format!("{tag}/00"), gui, 1, 32)?;
                ensure_unique(fields, &format!("merchant account {tag}"))?;
                for field in fields {
                    if field.tag == "00" {
                        return Err(Error::DuplicateTag(format!("{tag}/00")));
                    }
                    ascii_field(&format!("{tag}/{}", field.tag), &field.value, 1, 99)?;
                }
                Ok(())
            }
        }
    }

    fn to_tlv(&self) -> Result<Tlv> {
        match self {
            Self::Primitive { tag, value } => Tlv::new(tag.clone(), value.clone()),
            Self::Template { tag, gui, fields } => {
                let mut children = Vec::with_capacity(fields.len() + 1);
                children.push(Tlv::new("00", gui.clone())?);
                children.extend(fields.iter().cloned());
                Tlv::new(tag.clone(), encode_all(&children))
            }
        }
    }
}

/// Tag 62, the additional data field template.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdditionalData {
    /// 01: invoice or bill number.
    pub bill_number: Option<String>,
    /// 02: mobile number.
    pub mobile_number: Option<String>,
    /// 03: store label.
    pub store_label: Option<String>,
    /// 04: loyalty number.
    pub loyalty_number: Option<String>,
    /// 05: reference label identifying the transaction.
    pub reference_label: Option<String>,
    /// 06: customer label.
    pub customer_label: Option<String>,
    /// 07: terminal label.
    pub terminal_label: Option<String>,
    /// 08: purpose of the transaction.
    pub purpose: Option<String>,
    /// Every other sub-tag, kept as received.
    pub other: Vec<Tlv>,
}

impl AdditionalData {
    fn named(&self) -> [(&'static str, &Option<String>); 8] {
        [
            ("01", &self.bill_number),
            ("02", &self.mobile_number),
            ("03", &self.store_label),
            ("04", &self.loyalty_number),
            ("05", &self.reference_label),
            ("06", &self.customer_label),
            ("07", &self.terminal_label),
            ("08", &self.purpose),
        ]
    }

    fn parse(value: &str) -> Result<Self> {
        let children = parse_all(value)?;
        ensure_unique(&children, "additional data")?;
        let mut data = Self::default();
        for child in children {
            let slot = match child.tag.as_str() {
                "01" => &mut data.bill_number,
                "02" => &mut data.mobile_number,
                "03" => &mut data.store_label,
                "04" => &mut data.loyalty_number,
                "05" => &mut data.reference_label,
                "06" => &mut data.customer_label,
                "07" => &mut data.terminal_label,
                "08" => &mut data.purpose,
                _ => {
                    data.other.push(child);
                    continue;
                }
            };
            *slot = Some(child.value);
        }
        Ok(data)
    }

    fn validate(&self) -> Result<()> {
        for (tag, value) in self.named() {
            if let Some(value) = value {
                ascii_field(&format!("62/{tag}"), value, 1, 25)?;
            }
        }
        ensure_unique(&self.other, "additional data")?;
        for field in &self.other {
            if tag_number(&field.tag) <= 8 {
                return Err(Error::DuplicateTag(format!("62/{}", field.tag)));
            }
            ascii_field(&format!("62/{}", field.tag), &field.value, 0, 99)?;
        }
        Ok(())
    }

    fn to_tlv(&self) -> Result<Tlv> {
        let mut children = Vec::with_capacity(8 + self.other.len());
        for (tag, value) in self.named() {
            if let Some(value) = value {
                children.push(Tlv::new(tag, value.clone())?);
            }
        }
        children.extend(self.other.iter().cloned());
        children.sort_by(|a, b| a.tag.cmp(&b.tag));
        Tlv::new("62", encode_all(&children))
    }
}

/// Tag 64: merchant name and city in the merchant's own language and script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageTemplate {
    /// 00: ISO 639-1 language code, e.g. `et`.
    pub language: String,
    /// 01: merchant name, any Unicode characters, up to 25.
    pub merchant_name: String,
    /// 02: merchant city, up to 15 characters.
    pub merchant_city: Option<String>,
}

impl LanguageTemplate {
    fn parse(value: &str) -> Result<Self> {
        let children = parse_all(value)?;
        ensure_unique(&children, "language template")?;
        let get = |tag: &str| {
            children
                .iter()
                .find(|c| c.tag == tag)
                .map(|c| c.value.clone())
        };
        Ok(Self {
            language: get("00").ok_or_else(|| Error::MissingTag("64/00".into()))?,
            merchant_name: get("01").ok_or_else(|| Error::MissingTag("64/01".into()))?,
            merchant_city: get("02"),
        })
    }

    fn validate(&self) -> Result<()> {
        if self.language.len() != 2 || !self.language.bytes().all(|b| b.is_ascii_alphabetic()) {
            return Err(Error::InvalidPayload(format!(
                "invalid language preference {:?}",
                self.language
            )));
        }
        unicode_field("64/01", &self.merchant_name, 25)?;
        if let Some(city) = &self.merchant_city {
            unicode_field("64/02", city, 15)?;
        }
        Ok(())
    }

    fn to_tlv(&self) -> Result<Tlv> {
        let mut children = vec![
            Tlv::new("00", self.language.clone())?,
            Tlv::new("01", self.merchant_name.clone())?,
        ];
        if let Some(city) = &self.merchant_city {
            children.push(Tlv::new("02", city.clone())?);
        }
        Tlv::new("64", encode_all(&children))
    }
}

/// A Merchant-Presented Mode payload as typed fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MpmData {
    /// 01; absent means the scanner treats it as unspecified.
    pub point_of_initiation: Option<PointOfInitiation>,
    /// 02–51; at least one is required.
    pub merchant_accounts: Vec<MerchantAccount>,
    /// 52: ISO 18245 merchant category code, four digits.
    pub merchant_category_code: String,
    /// 53: ISO 4217 numeric currency code.
    pub currency_numeric: String,
    /// 54: absent means the payer enters the amount.
    pub amount: Option<String>,
    /// 58: ISO 3166-1 alpha-2 country code.
    pub country_code: String,
    /// 59: printable ASCII, up to 25 characters.
    pub merchant_name: String,
    /// 60: printable ASCII, up to 15 characters.
    pub merchant_city: String,
    /// 61: up to 10 characters.
    pub postal_code: Option<String>,
    pub additional_data: Option<AdditionalData>,
    pub language: Option<LanguageTemplate>,
    /// Tags 55–57 and 65–99, kept as received.
    pub other: Vec<Tlv>,
}

impl MpmData {
    /// Validates framing and checksum, then every per-tag rule. Tag order is
    /// not enforced beyond PFI first and CRC last.
    pub fn parse(payload: &str) -> Result<Self> {
        let objects = validate_mpm(payload)?;
        ensure_unique(&objects, "payload")?;

        let mut point_of_initiation = None;
        let mut merchant_accounts = Vec::new();
        let mut mcc = None;
        let mut currency = None;
        let mut amount = None;
        let mut country = None;
        let mut name = None;
        let mut city = None;
        let mut postal_code = None;
        let mut additional_data = None;
        let mut language = None;
        let mut other = Vec::new();

        for object in objects.into_iter().skip(1) {
            let n = tag_number(&object.tag);
            match n {
                1 => point_of_initiation = Some(PointOfInitiation::from_code(&object.value)?),
                2..=51 => merchant_accounts.push(MerchantAccount::parse(object.tag, object.value)?),
                52 => mcc = Some(object.value),
                53 => currency = Some(object.value),
                54 => amount = Some(object.value),
                58 => country = Some(object.value),
                59 => name = Some(object.value),
                60 => city = Some(object.value),
                61 => postal_code = Some(object.value),
                62 => additional_data = Some(AdditionalData::parse(&object.value)?),
                64 => language = Some(LanguageTemplate::parse(&object.value)?),
                _ => other.push(object),
            }
        }

        let data = Self {
            point_of_initiation,
            merchant_accounts,
            merchant_category_code: mcc.ok_or_else(|| Error::MissingTag("52".into()))?,
            currency_numeric: currency.ok_or_else(|| Error::MissingTag("53".into()))?,
            amount,
            country_code: country.ok_or_else(|| Error::MissingTag("58".into()))?,
            merchant_name: name.ok_or_else(|| Error::MissingTag("59".into()))?,
            merchant_city: city.ok_or_else(|| Error::MissingTag("60".into()))?,
            postal_code,
            additional_data,
            language,
            other,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<()> {
        if self.merchant_accounts.is_empty() {
            return Err(Error::MissingTag(
                "02-51 (merchant account information)".into(),
            ));
        }
        let mut seen = HashSet::with_capacity(self.merchant_accounts.len());
        for account in &self.merchant_accounts {
            if !seen.insert(account.tag()) {
                return Err(Error::DuplicateTag(account.tag().to_owned()));
            }
            account.validate()?;
        }

        digits_field("52", &self.merchant_category_code, 4)?;
        digits_field("53", &self.currency_numeric, 3)?;
        if let Some(amount) = &self.amount {
            iso4217::validate_amount(amount, iso4217::by_numeric(&self.currency_numeric))?;
        }
        if self.country_code.len() != 2
            || !self.country_code.bytes().all(|b| b.is_ascii_uppercase())
        {
            return Err(Error::InvalidPayload(format!(
                "invalid country code {:?}",
                self.country_code
            )));
        }
        ascii_field("59", &self.merchant_name, 1, 25)?;
        ascii_field("60", &self.merchant_city, 1, 15)?;
        if let Some(postal_code) = &self.postal_code {
            ascii_field("61", postal_code, 1, 10)?;
        }
        if let Some(data) = &self.additional_data {
            data.validate()?;
        }
        if let Some(language) = &self.language {
            language.validate()?;
        }

        ensure_unique(&self.other, "payload")?;
        for field in &self.other {
            let n = tag_number(&field.tag);
            if !matches!(n, 55..=57 | 65..=99) {
                return Err(Error::InvalidTag(field.tag.clone()));
            }
            if (55..=57).contains(&n) {
                ascii_field(&field.tag, &field.value, 1, 13)?;
            }
        }
        Ok(())
    }

    /// Validates, then encodes every object in ascending tag order followed by
    /// the CRC object.
    pub fn to_payload(&self) -> Result<MpmPayload> {
        self.validate()?;

        let mut objects = Vec::with_capacity(12 + self.merchant_accounts.len() + self.other.len());
        objects.push(Tlv::new("00", "01")?);
        if let Some(poi) = self.point_of_initiation {
            objects.push(Tlv::new("01", poi.code())?);
        }
        for account in &self.merchant_accounts {
            objects.push(account.to_tlv()?);
        }
        objects.push(Tlv::new("52", self.merchant_category_code.clone())?);
        objects.push(Tlv::new("53", self.currency_numeric.clone())?);
        if let Some(amount) = &self.amount {
            objects.push(Tlv::new("54", amount.clone())?);
        }
        objects.push(Tlv::new("58", self.country_code.clone())?);
        objects.push(Tlv::new("59", self.merchant_name.clone())?);
        objects.push(Tlv::new("60", self.merchant_city.clone())?);
        if let Some(postal_code) = &self.postal_code {
            objects.push(Tlv::new("61", postal_code.clone())?);
        }
        if let Some(data) = &self.additional_data {
            objects.push(data.to_tlv()?);
        }
        if let Some(language) = &self.language {
            objects.push(language.to_tlv()?);
        }
        objects.extend(self.other.iter().cloned());
        objects.sort_by(|a, b| a.tag.cmp(&b.tag));

        let mut body = encode_all(&objects);
        body.push_str("6304");
        let checksum = crc_hex(body.as_bytes());
        body.push_str(&checksum);

        if body.chars().count() > MAX_PAYLOAD_CHARS {
            return Err(Error::InvalidPayload(format!(
                "payload exceeds {MAX_PAYLOAD_CHARS} characters"
            )));
        }
        Ok(MpmPayload(body))
    }

    /// The merchant account template identified by `gui`, if present.
    pub fn merchant_account_by_gui(&self, gui: &str) -> Option<&MerchantAccount> {
        self.merchant_accounts
            .iter()
            .find(|a| matches!(a, MerchantAccount::Template { gui: g, .. } if g == gui))
    }
}

fn tag_number(tag: &str) -> u8 {
    tag.parse().unwrap_or(u8::MAX)
}

fn ensure_unique(objects: &[Tlv], scope: &str) -> Result<()> {
    let mut seen = HashSet::with_capacity(objects.len());
    for object in objects {
        if !seen.insert(object.tag.as_str()) {
            return Err(Error::DuplicateTag(format!("{scope}: {}", object.tag)));
        }
    }
    Ok(())
}

fn ascii_field(tag: &str, value: &str, min: usize, max: usize) -> Result<()> {
    if !is_printable_ascii(value) {
        return Err(Error::InvalidPayload(format!(
            "tag {tag} must be printable ASCII"
        )));
    }
    if value.len() < min || value.len() > max {
        return Err(Error::InvalidLength {
            tag: tag.to_owned(),
            length: value.len(),
        });
    }
    Ok(())
}

fn unicode_field(tag: &str, value: &str, max: usize) -> Result<()> {
    let length = value.chars().count();
    if length == 0 || length > max || value.chars().any(char::is_control) {
        return Err(Error::InvalidLength {
            tag: tag.to_owned(),
            length,
        });
    }
    Ok(())
}

fn digits_field(tag: &str, value: &str, len: usize) -> Result<()> {
    if value.len() != len || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidPayload(format!(
            "tag {tag} must be {len} digits"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MpmData {
        MpmData {
            point_of_initiation: Some(PointOfInitiation::Dynamic),
            merchant_accounts: vec![MerchantAccount::Template {
                tag: "26".into(),
                gui: "com.example.pay".into(),
                fields: vec![Tlv::new("01", "abc123").unwrap()],
            }],
            merchant_category_code: "5812".into(),
            currency_numeric: "978".into(),
            amount: Some("25.5".into()),
            country_code: "EE".into(),
            merchant_name: "Oismae Kohvik".into(),
            merchant_city: "Tallinn".into(),
            postal_code: None,
            additional_data: Some(AdditionalData {
                bill_number: Some("ORD42".into()),
                reference_label: Some("abc123".into()),
                ..Default::default()
            }),
            language: Some(LanguageTemplate {
                language: "et".into(),
                merchant_name: "Õismäe Kohvik".into(),
                merchant_city: None,
            }),
            other: Vec::new(),
        }
    }

    #[test]
    fn round_trips_through_the_wire_format() {
        let data = sample();
        let payload = data.to_payload().unwrap();
        assert!(payload.starts_with("000201010212"));
        assert!(payload.contains("64230002et0113Õismäe Kohvik"), "{payload}");
        assert_eq!(MpmData::parse(&payload).unwrap(), data);
    }

    #[test]
    fn rejects_tampering() {
        let payload = sample().to_payload().unwrap().into_inner();
        let tampered = payload.replace("25.5", "95.5");
        assert!(matches!(
            MpmData::parse(&tampered),
            Err(Error::CrcMismatch { .. })
        ));
    }

    #[test]
    fn enforces_field_rules() {
        let cases: [fn(&mut MpmData); 16] = [
            (|d| d.merchant_accounts.clear()),
            (|d| d.merchant_category_code = "58120".into()),
            (|d| d.currency_numeric = "EUR".into()),
            (|d| d.amount = Some("25.555".into())),
            (|d| d.country_code = "ee".into()),
            (|d| d.merchant_name = "a".repeat(26)),
            (|d| d.merchant_name = "Õismäe".into()),
            (|d| d.merchant_city = "a".repeat(16)),
            (|d| d.postal_code = Some("12345678901".into())),
            (|d| d.additional_data.as_mut().unwrap().bill_number = Some("a".repeat(26))),
            (|d| d.language.as_mut().unwrap().language = "est".into()),
            (|d| d.language.as_mut().unwrap().merchant_name = "ä".repeat(26)),
            (|d| d.other.push(Tlv::new("58", "EE").unwrap())),
            (|d| {
                d.merchant_accounts.push(MerchantAccount::Template {
                    tag: "26".into(),
                    gui: "x".into(),
                    fields: Vec::new(),
                })
            }),
            (|d| {
                d.merchant_accounts.push(MerchantAccount::Primitive {
                    tag: "27".into(),
                    value: "x".into(),
                })
            }),
            (|d| {
                d.merchant_accounts[0] = MerchantAccount::Template {
                    tag: "26".into(),
                    gui: "a".repeat(33),
                    fields: Vec::new(),
                }
            }),
        ];
        for (i, mutate) in cases.iter().enumerate() {
            let mut data = sample();
            mutate(&mut data);
            assert!(data.to_payload().is_err(), "case {i} should be rejected");
        }
    }

    #[test]
    fn parses_out_of_order_tags() {
        let objects = [
            Tlv::new("00", "01").unwrap(),
            Tlv::new("58", "EE").unwrap(),
            Tlv::new("26", "0015com.example.pay").unwrap(),
            Tlv::new("53", "978").unwrap(),
            Tlv::new("52", "4829").unwrap(),
            Tlv::new("60", "Tallinn").unwrap(),
            Tlv::new("59", "Mari K").unwrap(),
        ];
        let mut body = encode_all(&objects);
        body.push_str("6304");
        let checksum = crc_hex(body.as_bytes());
        body.push_str(&checksum);
        let data = MpmData::parse(&body).unwrap();
        assert_eq!(data.merchant_name, "Mari K");
        assert!(data.merchant_account_by_gui("com.example.pay").is_some());
    }

    #[test]
    fn rejects_duplicate_and_missing_tags() {
        let build = |objects: &[Tlv]| {
            let mut body = encode_all(objects);
            body.push_str("6304");
            let checksum = crc_hex(body.as_bytes());
            body + &checksum
        };
        let base = [
            Tlv::new("00", "01").unwrap(),
            Tlv::new("26", "0015com.example.pay").unwrap(),
            Tlv::new("52", "4829").unwrap(),
            Tlv::new("53", "978").unwrap(),
            Tlv::new("58", "EE").unwrap(),
            Tlv::new("59", "Mari K").unwrap(),
        ];
        assert!(matches!(
            MpmData::parse(&build(&base)),
            Err(Error::MissingTag(t)) if t == "60"
        ));
        let mut duplicated = base.to_vec();
        duplicated.push(Tlv::new("58", "FI").unwrap());
        assert!(matches!(
            MpmData::parse(&build(&duplicated)),
            Err(Error::DuplicateTag(_))
        ));
    }

    #[test]
    fn rejects_oversized_payloads() {
        let mut data = sample();
        for tag in 80..=86u8 {
            data.other
                .push(Tlv::new(tag.to_string(), "x".repeat(70)).unwrap());
        }
        assert!(data.to_payload().is_err());
    }
}
