
use slint::Color as SlintColor;

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
        let border_c = c.clamp(0.3, 0.4);
        border = SlintColor::from_oklch(border_l, border_c, h, alpha);
    }

    (preview, border)
}
