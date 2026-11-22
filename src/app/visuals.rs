
use slint::Color as SlintColor;
use crate::core::color::Color;

pub fn make_drag_preview(
    width: f32,
    height: f32,
    hex: Option<&str>,
) -> slint::Image {
    let w = width.round() as u32;
    let h = height.round() as u32;

    let (r, g, b) = hex
        .and_then(|s| Color::from_hex(s).ok())
        .map(|c| (c.red(), c.green(), c.blue()))
        .unwrap_or((124, 114, 232));

    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
    let slice = buf.make_mut_slice();

    for p in slice {
        *p = slint::Rgba8Pixel {
            r,
            g,
            b,
            a: 255,
        };
    }

    slint::Image::from_rgba8_premultiplied(buf)
}

pub fn calculate_drag_colors(original: SlintColor) -> (SlintColor, SlintColor) {
    let oklch = original.to_oklch();
    let l = oklch.lightness;
    let c = oklch.chroma;
    let h = oklch.hue;
    let alpha = oklch.alpha;
    
    let preview;
    let border;
    
    // in case ur wondering, 1e-3 = 1 * 10^-3 = 0.001
    // means if the color is basically colorless (for example shades of grey)
    if c < 1e-3 {
        let new_l = (l * 0.8 + 0.1).clamp(0.0, 1.0);
        let border_l = 1.0 - l;

        preview = SlintColor::from_oklch(new_l, 0.0, 0.0, alpha);
        border  = SlintColor::from_oklch(border_l, 0.0, 0.0, alpha);
    } else {
        let new_l = (l * 0.8 + 0.1).clamp(0.0, 1.0);
        let new_c = (c * 0.62).clamp(0.0, 0.4);
        preview = SlintColor::from_oklch(new_l, new_c, h, alpha);

        let border_l = 1.0;
        let border_c = c.max(0.3).min(0.4);
        border = SlintColor::from_oklch(border_l, border_c, h, alpha);
    }

    (preview, border)
}
