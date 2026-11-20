use std::cell::Cell;

use fast_qr::{
    convert::{svg::SvgBuilder, Builder, Shape},
    qr::QRBuilder,
    Module, ECL,
};
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{self, Tree};

use crate::{Error, Result};

const MARGIN: usize = 4;
const INTER_FONT_BYTES: &[u8] = include_bytes!("fonts/Inter-Bold.ttf");

thread_local! {
    static QR_SIZE: Cell<usize> = const { Cell::new(0) };
    static TEXT_BOUNDS: Cell<Option<(f64, f64, f64, f64)>> = const { Cell::new(None) };
}

#[inline]
fn is_corner(module: Module) -> bool {
    // In fast_qr, ModuleType::FinderPattern is (1 << 1), so (module.0 >> 1) == 1
    (module.0 >> 1) == 1
}

#[inline]
fn get_finder_coords(y: usize, x: usize, size: usize) -> Option<(usize, usize)> {
    if y < MARGIN || x < MARGIN {
        return None;
    }
    let qr_y = y - MARGIN;
    let qr_x = x - MARGIN;

    if qr_y < 7 && qr_x < 7 {
        Some((qr_y, qr_x))
    } else if qr_y < 7 && qr_x + 7 >= size {
        Some((qr_y, qr_x - (size - 7)))
    } else if qr_y + 7 >= size && qr_x < 7 {
        Some((qr_y - (size - 7), qr_x))
    } else {
        None
    }
}

fn round_corner_module(y: usize, x: usize, ry: usize, rx: usize) -> String {
    // 4 outer corners of the 7x7 finder frame
    // and 4 corners of the inner 3x3 square block
    let is_tl = (ry == 0 && rx == 0) || (ry == 2 && rx == 2);
    let is_tr = (ry == 0 && rx == 6) || (ry == 2 && rx == 4);
    let is_br = (ry == 6 && rx == 6) || (ry == 4 && rx == 4);
    let is_bl = (ry == 6 && rx == 0) || (ry == 4 && rx == 2);

    if is_tl {
        return format!("M{},{y}v1h-1a1,1 0 0 1 1,-1z", x + 1);
    }
    if is_tr {
        return format!("M{x},{y}a1,1 0 0 1 1,1h-1v-1z");
    }
    if is_br {
        return format!("M{},{y}a1,1 0 0 1 -1,1v-1h1z", x + 1);
    }
    if is_bl {
        return format!("M{x},{y}h1v1a1,1 0 0 1 -1,-1z");
    }

    format!("M{x},{y}h1v1h-1")
}

#[inline]
fn circle_module_path(y: usize, x: usize) -> String {
    format!("M{},{y}.5a.5,.5 0 1,1 0,-.1", x + 1)
}

fn qr_module_render(y: usize, x: usize, module: Module) -> String {
    let size = QR_SIZE.get();
    if is_corner(module) {
        if let Some((ry, rx)) = get_finder_coords(y, x, size) {
            return round_corner_module(y, x, ry, rx);
        }
        return format!("M{x},{y}h1v1h-1");
    }

    if let Some((min_x, min_y, max_x, max_y)) = TEXT_BOUNDS.get() {
        let fx = x as f64;
        let fy = y as f64;
        if fx + 1.0 > min_x && fx < max_x && fy + 1.0 > min_y && fy < max_y {
            return String::new();
        }
    }

    circle_module_path(y, x)
}

/// Renders a QR code as PNG bytes, with circular data dots and rounded finder
/// squares. `center_text` (e.g. "pay.") adds a badge and raises error
/// correction to level H so the code still scans.
pub fn render_png(payload: &str, width: u32, center_text: Option<&str>) -> Result<Vec<u8>> {
    let mut builder = QRBuilder::new(payload);
    if center_text.is_some() {
        builder.ecl(ECL::H);
    }
    let qr = builder.build().map_err(|e| Error::Qr(format!("{e:?}")))?;

    QR_SIZE.set(qr.size);
    if let Some(text) = center_text {
        let total_grid = (qr.size + MARGIN * 2) as f64;
        let center = total_grid / 2.0;
        let char_count = text.chars().count() as f64;
        let cutout_w = (char_count * 4.4 + 4.0).max(16.0);
        let cutout_h = 10.0;
        let min_x = center - cutout_w / 2.0;
        let max_x = center + cutout_w / 2.0;
        let min_y = center - cutout_h / 2.0;
        let max_y = center + cutout_h / 2.0;
        TEXT_BOUNDS.set(Some((min_x, min_y, max_x, max_y)));
    } else {
        TEXT_BOUNDS.set(None);
    }

    let mut svg_builder = SvgBuilder::default();
    svg_builder.shape(Shape::Command(qr_module_render));
    svg_builder.margin(MARGIN);
    let mut svg = svg_builder.to_str(&qr);
    QR_SIZE.set(0);
    TEXT_BOUNDS.set(None);

    if let Some(text) = center_text {
        let total_grid = (qr.size + MARGIN * 2) as f64;
        let center = total_grid / 2.0;
        let font_size = 8.4;
        let text = escape_xml(text);

        let badge = format!(
            r##"<g id="center-badge">
                <text x="{center:.2}" y="{center:.2}" font-family="Inter" font-size="{font_size:.2}" font-weight="bold" text-anchor="middle" dominant-baseline="central" fill="#111111">{text}</text>
            </g></svg>"##
        );

        svg = svg.replace("</svg>", &badge);
    }

    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_font_data(INTER_FONT_BYTES.to_vec());
    opt.font_family = "Inter".into();

    let tree = Tree::from_str(&svg, &opt).map_err(|e| Error::Qr(format!("{e:?}")))?;
    let factor = width as f32 / tree.size().width();
    let w = (tree.size().width() * factor) as u32;
    let h = (tree.size().height() * factor) as u32;

    let mut pixmap =
        Pixmap::new(w, h).ok_or_else(|| Error::Qr("failed to allocate pixmap".into()))?;
    resvg::render(
        &tree,
        usvg::Transform::from_scale(factor, factor),
        &mut pixmap.as_mut(),
    );

    pixmap.encode_png().map_err(|e| Error::Qr(format!("{e:?}")))
}

/// Renders a QR code to a PNG file. See [`render_png`].
pub fn write_png(payload: &str, path: &str, width: u32) -> Result<()> {
    write_png_with_center_text(payload, path, width, None)
}

/// Renders a QR code with an optional center badge to a PNG file. See [`render_png`].
pub fn write_png_with_center_text(
    payload: &str,
    path: &str,
    width: u32,
    center_text: Option<&str>,
) -> Result<()> {
    std::fs::write(path, render_png(payload, width, center_text)?)?;
    Ok(())
}

fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}
