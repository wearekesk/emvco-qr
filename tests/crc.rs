use emvco_qr::crc::crc_hex;

#[test]
fn crc16_ccitt_false_check_value() {
    assert_eq!(crc_hex(b"123456789"), "29B1");
}
