/// CRC-16/CCITT-FALSE:
/// polynomial 0x1021, initial value 0xFFFF, no reflection, xorout 0x0000.
pub fn crc16_ccitt_false(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;

    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            crc = if (crc & 0x8000) != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }

    crc
}

pub fn crc_hex(data: &[u8]) -> String {
    format!("{:04X}", crc16_ccitt_false(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_crc_check_value() {
        assert_eq!(crc_hex(b"123456789"), "29B1");
    }
}
