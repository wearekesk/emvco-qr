use emvco_qr::{
    iso4217,
    mpm::{AdditionalData, LanguageTemplate, MerchantAccount, MpmData, PointOfInitiation},
    qr_image::{write_png, write_png_with_center_text},
    text::transliterate_ascii,
    tlv::Tlv,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eur = iso4217::by_alpha("EUR").ok_or("EUR missing from the ISO 4217 table")?;
    let name = "Õismäe Kohvik";

    // A scheme's own merchant account template: sub-tag 00 is its globally
    // unique identifier, the rest are whatever that scheme defines.
    let account = MerchantAccount::Template {
        tag: "26".into(),
        gui: "com.example.pay".into(),
        fields: vec![Tlv::new("01", "req98a7f14b2c")?],
    };

    let payload = MpmData {
        point_of_initiation: Some(PointOfInitiation::Dynamic),
        merchant_accounts: vec![account],
        merchant_category_code: "5812".into(),
        currency_numeric: eur.numeric.into(),
        amount: Some(iso4217::format_amount(2550, eur)?),
        country_code: "EE".into(),
        merchant_name: transliterate_ascii(name),
        merchant_city: "Tallinn".into(),
        postal_code: None,
        additional_data: Some(AdditionalData {
            bill_number: Some("ORD42".into()),
            ..Default::default()
        }),
        language: Some(LanguageTemplate {
            language: "et".into(),
            merchant_name: name.into(),
            merchant_city: None,
        }),
        other: Vec::new(),
    }
    .to_payload()?;

    println!("Generated EMVCo MPM payload:\n{payload}");

    std::fs::create_dir_all("target")?;
    write_png(&payload, "target/sample_qr.png", 512)?;
    write_png_with_center_text(&payload, "target/sample_qr_badge.png", 512, Some("pay."))?;
    println!("Saved target/sample_qr.png and target/sample_qr_badge.png");

    Ok(())
}
