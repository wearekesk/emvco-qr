use emvco_qr::{
    mpm::MpmBuilder,
    qr_image::{write_png, write_png_with_center_text},
};

#[test]
fn generates_qr_image_png() {
    let payload = MpmBuilder::new()
        .field("00", "01")
        .unwrap()
        .field("58", "EE")
        .unwrap()
        .finish()
        .unwrap();

    let temp_dir = std::env::temp_dir();
    let qr_path = temp_dir.join("test_emvco_qr.png");
    let path_str = qr_path.to_str().unwrap();

    let res = write_png(&payload, path_str, 256);
    assert!(res.is_ok());

    let bytes = std::fs::read(&qr_path).unwrap();
    // Verify PNG magic header
    assert!(bytes.len() > 8);
    assert_eq!(
        &bytes[0..8],
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
    );

    let _ = std::fs::remove_file(qr_path);
}

#[test]
fn generates_qr_image_with_center_text() {
    let payload = MpmBuilder::new()
        .field("00", "01")
        .unwrap()
        .field("58", "EE")
        .unwrap()
        .finish()
        .unwrap();

    let temp_dir = std::env::temp_dir();
    let qr_path = temp_dir.join("test_emvco_qr_badge.png");
    let path_str = qr_path.to_str().unwrap();

    let res = write_png_with_center_text(&payload, path_str, 256, Some("pay."));
    assert!(res.is_ok());

    let bytes = std::fs::read(&qr_path).unwrap();
    assert!(bytes.len() > 8);
    assert_eq!(
        &bytes[0..8],
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
    );

    let _ = std::fs::remove_file(qr_path);
}

#[test]
fn render_png_returns_bytes_and_escapes_badge_text() {
    let payload = MpmBuilder::new()
        .field("00", "01")
        .unwrap()
        .field("58", "EE")
        .unwrap()
        .finish()
        .unwrap();

    let bytes = emvco_qr::qr_image::render_png(&payload, 256, Some("a<&>b")).unwrap();
    assert_eq!(&bytes[0..4], &[0x89, b'P', b'N', b'G']);
}
