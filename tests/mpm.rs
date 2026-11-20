use emvco_qr::{
    mpm::{validate_mpm, MpmBuilder},
    profiles::GenericProfile,
    tlv::Tlv,
};

#[test]
fn builder_generates_crc_that_validator_accepts() {
    let payload = MpmBuilder::new()
        .field("00", "01")
        .unwrap()
        .field("01", "11")
        .unwrap()
        .field("52", "0000")
        .unwrap()
        .field("53", "978")
        .unwrap()
        .field("58", "EE")
        .unwrap()
        .finish()
        .unwrap();

    assert!(validate_mpm(&payload.0).is_ok());
}

#[test]
fn detects_modified_crc() {
    let payload = MpmBuilder::new()
        .field("00", "01")
        .unwrap()
        .field("58", "EE")
        .unwrap()
        .finish()
        .unwrap();

    let mut tampered = payload.0;
    let last = tampered.pop().unwrap();
    tampered.push(if last == '0' { '1' } else { '0' });

    assert!(validate_mpm(&tampered).is_err());
}

#[test]
fn builder_with_a_scheme_template() {
    let child1 = Tlv::new("00", "com.example.pay").unwrap();
    let child2 = Tlv::new("01", "req_12345").unwrap();

    let payload = MpmBuilder::with_profile(GenericProfile)
        .field("00", "01")
        .unwrap()
        .template("26", &[child1, child2])
        .unwrap()
        .finish()
        .unwrap();

    let objects = validate_mpm(&payload).unwrap();
    assert_eq!(objects.len(), 2);
    assert_eq!(objects[0].tag, "00");
    assert_eq!(objects[0].value, "01");
    assert_eq!(objects[1].tag, "26");
}

/// The sample payload from the EMVCo MPM specification: two scheme templates,
/// a tip indicator, additional data, a Chinese language template and an
/// unreserved template (91).
const EMVCO_SPEC_SAMPLE: &str = "00020101021229300012D156000000000510A93FO3230Q31280012D15600000001030812345678520441115802CN5914BEST TRANSPORT6007BEIJING64200002ZH0104最佳运输0202北京540523.7253031565502016233030412340603***0708A60086670902ME91320016A0112233449988770708123456786304A13A";

#[test]
fn parses_the_emvco_specification_sample() {
    use emvco_qr::mpm::{MerchantAccount, MpmData, PointOfInitiation};

    let data = MpmData::parse(EMVCO_SPEC_SAMPLE).unwrap();
    assert_eq!(data.point_of_initiation, Some(PointOfInitiation::Dynamic));
    assert_eq!(data.merchant_category_code, "4111");
    assert_eq!(data.currency_numeric, "156");
    assert_eq!(data.amount.as_deref(), Some("23.72"));
    assert_eq!(data.country_code, "CN");
    assert_eq!(data.merchant_name, "BEST TRANSPORT");
    assert_eq!(data.merchant_city, "BEIJING");
    let language = data.language.as_ref().unwrap();
    assert_eq!(language.merchant_name, "最佳运输");
    assert_eq!(language.merchant_city.as_deref(), Some("北京"));
    let extra = data.additional_data.as_ref().unwrap();
    assert_eq!(extra.store_label.as_deref(), Some("1234"));
    assert_eq!(extra.terminal_label.as_deref(), Some("A6008667"));
    assert!(matches!(
        &data.merchant_accounts[0],
        MerchantAccount::Template { tag, gui, .. } if tag == "29" && gui == "D15600000000"
    ));

    // Re-encoding orders tags ascending, so the bytes differ but the data does not.
    let reencoded = data.to_payload().unwrap();
    assert_eq!(MpmData::parse(&reencoded).unwrap(), data);
}

#[test]
fn rejects_any_single_character_change_to_the_sample() {
    use emvco_qr::mpm::MpmData;

    let body: Vec<char> = EMVCO_SPEC_SAMPLE.chars().collect();
    for i in 0..body.len() - 8 {
        let mut mutated = body.clone();
        mutated[i] = if mutated[i] == '1' { '2' } else { '1' };
        let mutated: String = mutated.into_iter().collect();
        assert!(MpmData::parse(&mutated).is_err(), "change at {i} accepted");
    }
}
