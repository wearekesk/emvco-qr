use emvco_qr::cpm::{ber_tlv::BerTlv, builder::encode_qr_payload, validate::decode_qr_payload};

#[test]
fn cpm_base64_round_trip_for_prevalidated_bytes() {
    let bytes = [0x5A, 0x01, 0xFF];
    let encoded = encode_qr_payload(&bytes).unwrap();
    assert_eq!(decode_qr_payload(&encoded).unwrap(), bytes);
}

#[test]
fn cpm_ber_tlv_round_trip() {
    let tlv = BerTlv::new(
        vec![0x61],
        vec![0x4F, 0x07, 0xA0, 0x00, 0x00, 0x00, 0x04, 0x10, 0x10],
    );
    let encoded = tlv.encode().unwrap();
    assert_eq!(encoded[0], 0x61);
    assert_eq!(encoded[1], 9);
    assert_eq!(
        &encoded[2..],
        &[0x4F, 0x07, 0xA0, 0x00, 0x00, 0x00, 0x04, 0x10, 0x10]
    );

    let qr_payload = encode_qr_payload(&encoded).unwrap();
    let decoded = decode_qr_payload(&qr_payload).unwrap();
    assert_eq!(decoded, encoded);
}
