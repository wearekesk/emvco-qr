# `emvco-qr`

Rust library for constructing, parsing, validating, and rendering EMVCo payment QR payloads. Scheme-specific rules are layered on top through profiles, kept apart from the generic EMVCo formats.

> **Status:** Implementation blueprint with working core examples. Do not describe this crate as fully EMVCo-conformant until the implementation has been checked against the current official MPM/CPM specifications and EMVCo self-evaluation vectors. In particular, CPM has a distinct BER-TLV payload model and must not be implemented as reversed MPM.

## Goals

- Separate EMVCo format mechanics from scheme-specific rules.
- Support Merchant-Presented Mode (MPM) and Consumer-Presented Mode (CPM) through separate modules.
- Provide QR image output using [`fast_qr`](https://docs.rs/fast_qr/).
- Validate payload lengths, tags, mandatory objects, and checksums where the mode defines them.
- Make payment QR codes identify a request; never treat possession of a QR code as authorization to debit a wallet.
- Keep external payment processing, wallet ledger posting, authentication, and settlement outside this crate.

## Standards and Interoperability

- **MPM (Merchant-Presented Mode):** Human-readable two-digit Tag-Length-Value (TLV) objects. The payload ends in tag `63`, length `04`, and a CRC-16/CCITT-FALSE value calculated over the payload including `6304` but excluding the four checksum characters.
- **CPM (Consumer-Presented Mode):** BER-TLV data objects. The QR encodes the Base64 representation of the CPM payload. It has its own mandatory Payload Format Indicator and application-template requirements. It does **not** use the MPM `6304` checksum convention.

## Usage

### Merchant-Presented Mode (MPM)

```rust
use emvco_qr::mpm::{MpmBuilder, validate_mpm};

let payload = MpmBuilder::new()
    .field("00", "01")?
    .field("01", "11")?
    .field("52", "0000")?
    .field("53", "978")?
    .field("58", "EE")?
    .finish()?;

// Validate MPM payload string
let objects = validate_mpm(&payload)?;
```

### Typed MPM payloads

`MpmData` parses a payload into typed fields and encodes one back, enforcing the
per-tag rules: formats and lengths, at least one merchant account, the
additional data template (62) and the language template (64). Parsing accepts
tags in any order; encoding writes them in ascending order. Lengths count
characters, the CRC covers the UTF-8 bytes, and only tag 64 may hold non-ASCII
text. Use `text::transliterate_ascii` for tags 59 and 60.

```rust
use emvco_qr::{iso4217, mpm::MpmData};

let data = MpmData::parse(&scanned)?;
let currency = iso4217::by_numeric(&data.currency_numeric);
```

### Scheme templates

A payment scheme identifies itself in a merchant account template (tags
26-51): sub-tag 00 is its globally unique identifier, and the other sub-tags
are whatever the scheme defines. Build one with `MerchantAccount::Template`
and find it again with `MpmData::merchant_account_by_gui`. Validating a
scheme's own sub-tags is the scheme's job, through an `MpmProfile`.

```rust
use emvco_qr::{mpm::{MerchantAccount, MpmData}, tlv::Tlv};

let account = MerchantAccount::Template {
    tag: "26".into(),
    gui: "com.example.pay".into(),
    fields: vec![Tlv::new("01", "order-42")?],
};
// ...put `account` in MpmData::merchant_accounts, then:
let parsed = MpmData::parse(&payload)?;
let reference = parsed
    .merchant_account_by_gui("com.example.pay")
    .and_then(|a| a.field("01"));
```

A QR code identifies a payment request; it is never proof of payment, and
an amount read from one should be checked against the payee's own record.

### Consumer-Presented Mode (CPM)

```rust
use emvco_qr::cpm::{builder::encode_qr_payload, validate::decode_qr_payload};

let bytes = [0x5A, 0x01, 0xFF];
let encoded = encode_qr_payload(&bytes)?;
let decoded = decode_qr_payload(&encoded)?;
```

### QR Image Rendering

Needs the `image` feature. Renders with circular data dots and solid corner squares with rounded edges. `render_png` returns the bytes:

```rust
use emvco_qr::qr_image::{write_png, write_png_with_center_text};

// Standard QR code
write_png(&payload, "target/qr.png", 512)?;

// QR code with a center text badge
write_png_with_center_text(&payload, "target/qr_badge.png", 512, Some("pay."))?;
```

## Testing

```bash
cargo test --all-features
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

## License

Licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option.
