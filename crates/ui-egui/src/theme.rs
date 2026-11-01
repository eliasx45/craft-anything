//! Design system: two themes (Studio dark, Studio Light), colour tokens, radii and type.
//!
//! A calm desktop look: neutral graphite (or near-white) surfaces, a slightly darker sidebar,
//! soft rounded cards and controls, sentence-case Inter throughout, and ink-filled main buttons.
//! Monochrome: grey fills step up in small increments and white (black in light) is the accent.
//! Widgets read [`Tokens::get`] instead of hard-coding colours, so a theme switch applies everywhere.
//! Confidence has its own fixed palette so "how sure are we" reads the same in both themes.

use craft_anything_engine::evidence::Confidence;
use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeKind {
    #[default]
    Studio,
    StudioLight,
}

impl ThemeKind {
    pub const ALL: [ThemeKind; 2] = [ThemeKind::Studio, ThemeKind::StudioLight];

    pub fn label(self) -> &'static str {
        match self {
            ThemeKind::Studio => "Studio (dark)",
            ThemeKind::StudioLight => "Studio Light",
        }
    }

    /// The value stored in `settings.theme`.
    pub fn id(self) -> &'static str {
        match self {
            ThemeKind::Studio => "studio",
            ThemeKind::StudioLight => "studio-light",
        }
    }

    pub fn from_id(s: &str) -> ThemeKind {
        match s.trim().to_ascii_lowercase().replace(['_', ' '], "-").as_str() {
            "studio-light" | "light" => ThemeKind::StudioLight,
            _ => ThemeKind::Studio,
        }
    }
}

/// Colour and shape tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub kind: ThemeKind,
    /// Title bar, rails, panels.
    pub chrome: Color32,
    /// The main reading surface behind cards.
    pub canvas: Color32,
    pub card: Color32,
    pub card_border: Color32,
    /// A card a step above `card` (hovered cards, chips).
    pub raised: Color32,
    pub field: Color32,
    pub field_border: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    pub icon: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    /// Indigo that reads as text on the theme's surfaces.
    pub accent_ink: Color32,
    /// Indigo wash for soft highlights.
    pub accent_soft: Color32,
    /// Text drawn on an `accent` fill.
    pub on_accent: Color32,
    pub separator: Color32,
    pub shadow: Color32,
    pub danger: Color32,
    pub warning: Color32,
    pub success: Color32,
    pub code_bg: Color32,
    /// The person's own messages.
    pub bubble_user: Color32,
    /// The app's and the builder's messages.
    pub bubble_agent: Color32,
    /// The floating prompt box.
    pub composer: Color32,
    /// Round controls inside the prompt box (attach, voice).
    pub composer_control: Color32,
    /// Floating chrome pills (the project title at the top of the feed).
    pub pill_fill: Color32,
    /// Hairline ring around outlined chrome (round header buttons, footer pills).
    pub chrome_stroke: Color32,
    /// Secondary buttons, switches that are off, avatar placeholders.
    pub secondary_fill: Color32,
    /// The chosen row in a list (the open project in the sidebar).
    pub selected_row: Color32,
    pub radius_sm: f32,
    pub radius: f32,
    pub radius_lg: f32,
    /// Floating surfaces: the prompt box, dialogs and sheets.
    pub radius_xl: f32,
}

/// The old app colour, kept for the web viewer and the confidence palette's neighbours. The
/// desktop themes are monochrome: their accent is white (dark) or black (light).
pub const BRAND: Color32 = Color32::from_rgb(0x16, 0x59, 0xc4);

pub const SEEN_IT: Color32 = Color32::from_rgb(0x3f, 0xb2, 0x7f);
pub const WORKED_OUT: Color32 = Color32::from_rgb(0x5b, 0x8d, 0xef);
pub const BEST_GUESS: Color32 = Color32::from_rgb(0xe0, 0xa2, 0x3a);
pub const UNKNOWN: Color32 = Color32::from_rgb(0x8a, 0x8a, 0x93);

/// The colour for a confidence level (`None` = unknown).
pub fn confidence_color(c: Option<Confidence>) -> Color32 {
    match c {
        Some(Confidence::Observed) => SEEN_IT,
        Some(Confidence::Derived) => WORKED_OUT,
        Some(Confidence::Inferred) => BEST_GUESS,
        None => UNKNOWN,
    }
}

pub fn confidence_label(c: Option<Confidence>) -> &'static str {
    c.map(Confidence::plain_label).unwrap_or("Unknown")
}

impl Tokens {
    pub fn for_kind(kind: ThemeKind) -> Self {
        match kind {
            // Monochrome dark: a near-black reading surface, a slightly lifted sidebar, grey fills
            // stepping up in small increments, and white as the only accent.
            ThemeKind::Studio => Tokens {
                kind,
                chrome: Color32::from_rgb(0x11, 0x11, 0x11),
                canvas: Color32::from_rgb(0x07, 0x07, 0x07),
                card: Color32::from_rgb(0x11, 0x11, 0x11),
                card_border: Color32::from_rgb(0x26, 0x26, 0x26),
                raised: Color32::from_rgb(0x1c, 0x1c, 0x1c),
                field: Color32::from_rgb(0x19, 0x19, 0x19),
                field_border: Color32::from_rgb(0x63, 0x63, 0x63),
                hover: Color32::from_rgb(0x1c, 0x1c, 0x1c),
                pressed: Color32::from_rgb(0x29, 0x29, 0x29),
                text: Color32::from_rgb(0xfa, 0xfa, 0xfa),
                text_dim: Color32::from_rgb(0xa0, 0xa0, 0xa0),
                text_faint: Color32::from_rgb(0x9a, 0x9a, 0x9a),
                icon: Color32::from_rgb(0xa0, 0xa0, 0xa0),
                accent: Color32::from_rgb(0xf5, 0xf5, 0xf5),
                accent_hover: Color32::WHITE,
                accent_ink: Color32::from_rgb(0xfa, 0xfa, 0xfa),
                accent_soft: Color32::from_rgba_unmultiplied(0xfa, 0xfa, 0xfa, 26),
                on_accent: Color32::BLACK,
                separator: Color32::from_rgb(0x26, 0x26, 0x26),
                shadow: Color32::from_black_alpha(110),
                danger: Color32::from_rgb(0xf0, 0xa7, 0xa7),
                warning: Color32::from_rgb(0xec, 0xaf, 0x52),
                success: Color32::from_rgb(0x8f, 0xd1, 0x8b),
                code_bg: Color32::from_rgb(0x11, 0x11, 0x11),
                bubble_user: Color32::from_rgb(0x36, 0x36, 0x36),
                bubble_agent: Color32::from_rgb(0x26, 0x26, 0x26),
                composer: Color32::from_rgb(0x2f, 0x2f, 0x2f),
                composer_control: Color32::from_rgb(0x41, 0x41, 0x41),
                pill_fill: Color32::from_rgb(0x18, 0x18, 0x18),
                chrome_stroke: Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 26),
                secondary_fill: Color32::from_rgb(0x3a, 0x3a, 0x3a),
                selected_row: Color32::from_rgb(0x29, 0x29, 0x29),
                radius_sm: 10.0,
                radius: 12.0,
                radius_lg: 16.0,
                radius_xl: 22.0,
            },
            // Monochrome light: white reading surface, a soft grey sidebar, black as the accent.
            ThemeKind::StudioLight => Tokens {
                kind,
                chrome: Color32::from_rgb(0xf4, 0xf4, 0xf4),
                canvas: Color32::WHITE,
                card: Color32::from_rgb(0xf4, 0xf4, 0xf4),
                card_border: Color32::from_rgb(0xe6, 0xe6, 0xe6),
                raised: Color32::from_rgb(0xf0, 0xf0, 0xf0),
                field: Color32::from_rgb(0xf4, 0xf4, 0xf4),
                field_border: Color32::from_rgb(0x8a, 0x8a, 0x8a),
                hover: Color32::from_rgb(0xef, 0xef, 0xef),
                pressed: Color32::from_rgb(0xe6, 0xe6, 0xe6),
                text: Color32::from_rgb(0x14, 0x14, 0x14),
                text_dim: Color32::from_rgb(0x62, 0x62, 0x62),
                text_faint: Color32::from_rgb(0x66, 0x66, 0x66),
                icon: Color32::from_rgb(0x5a, 0x5a, 0x5a),
                accent: Color32::from_rgb(0x0a, 0x0a, 0x0a),
                accent_hover: Color32::from_rgb(0x2a, 0x2a, 0x2a),
                accent_ink: Color32::from_rgb(0x14, 0x14, 0x14),
                accent_soft: Color32::from_rgba_unmultiplied(0x14, 0x14, 0x14, 20),
                on_accent: Color32::WHITE,
                separator: Color32::from_rgb(0xe6, 0xe6, 0xe6),
                shadow: Color32::from_black_alpha(26),
                danger: Color32::from_rgb(0xc2, 0x3a, 0x2b),
                warning: Color32::from_rgb(0x93, 0x60, 0x00),
                success: Color32::from_rgb(0x2a, 0x6e, 0x2e),
                code_bg: Color32::from_rgb(0xf4, 0xf4, 0xf4),
                bubble_user: Color32::from_rgb(0xe2, 0xe2, 0xe2),
                bubble_agent: Color32::from_rgb(0xee, 0xee, 0xee),
                composer: Color32::WHITE,
                composer_control: Color32::from_rgb(0xe9, 0xe9, 0xe9),
                pill_fill: Color32::from_rgb(0xf3, 0xf3, 0xf3),
                chrome_stroke: Color32::from_rgba_unmultiplied(0x00, 0x00, 0x00, 23),
                secondary_fill: Color32::from_rgb(0xe2, 0xe2, 0xe2),
                selected_row: Color32::from_rgb(0xe8, 0xe8, 0xe8),
                radius_sm: 10.0,
                radius: 12.0,
                radius_lg: 16.0,
                radius_xl: 22.0,
            },
        }
    }

    /// The tokens of the active theme.
    pub fn get(ctx: &egui::Context) -> Tokens {
        ctx.data(|d| d.get_temp::<Tokens>(tokens_id())).unwrap_or_else(|| Tokens::for_kind(ThemeKind::Studio))
    }

    pub fn dark(&self) -> bool {
        self.kind == ThemeKind::Studio
    }

    pub fn r_sm(&self) -> CornerRadius {
        CornerRadius::same(self.radius_sm as u8)
    }
    pub fn r(&self) -> CornerRadius {
        CornerRadius::same(self.radius as u8)
    }
    pub fn r_lg(&self) -> CornerRadius {
        CornerRadius::same(self.radius_lg as u8)
    }
    pub fn r_xl(&self) -> CornerRadius {
        CornerRadius::same(self.radius_xl as u8)
    }
    /// Fully round ends (pills, round buttons).
    pub fn r_pill(&self) -> CornerRadius {
        CornerRadius::same(u8::MAX)
    }
}

fn tokens_id() -> egui::Id {
    egui::Id::new("craft-anything-theme")
}

// ---- fonts -------------------------------------------------------------------------------------

pub static INTER_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
pub static INTER_MEDIUM: &[u8] = include_bytes!("../../../assets/fonts/Inter-Medium.ttf");
pub static INTER_SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf");
pub static JETBRAINS_MONO: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf");

/// A font from the optional craft-fonts build input (empty unless built with `CRAFT_FONTS_DIR`).
pub struct CraftFont {
    pub family: &'static str,
    pub style: &'static str,
    /// ISO 15924 scripts the font is for, e.g. `"Jpan"`.
    pub scripts: &'static [&'static str],
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/craft_fonts.rs"));

/// Install Inter (three weights: regular for text, medium for labels, semibold for headings
/// and buttons), JetBrains Mono (code), plus any craft-fonts as fallbacks.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut add = |name: &str, bytes: &'static [u8]| {
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    };
    add("Inter", INTER_REGULAR);
    add("Inter-Medium", INTER_MEDIUM);
    add("Inter-SemiBold", INTER_SEMIBOLD);
    add("JetBrainsMono", JETBRAINS_MONO);
    let mut fallbacks = Vec::new();
    for f in CRAFT_FONTS {
        let name = format!("craft:{}:{}", f.family, f.style);
        fonts.font_data.insert(name.clone(), Arc::new(FontData::from_static(f.bytes)));
        if f.style.eq_ignore_ascii_case("regular") {
            fallbacks.push(name);
        }
    }
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "Inter".to_owned());
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "JetBrainsMono".to_owned());
    for fam in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(fam).or_default().extend(fallbacks.iter().cloned());
    }
    let base: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (fam, primary) in [("medium", "Inter-Medium"), ("semibold", "Inter-SemiBold")] {
        let mut stack = vec![primary.to_owned()];
        stack.extend(base.iter().cloned());
        fonts.families.insert(FontFamily::Name(fam.into()), stack);
    }
    ctx.set_fonts(fonts);
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}
/// The type scale: every text size in the interface is one of these. It follows a calm desktop
/// ramp (26 · 20 · 15 · 13 · 12 · 11), a half point up where text is read at length.
/// Headings (`heading`, Inter semibold), largest first: a screen's title, a step or a dialog's
/// title, a card's or a list item's title.
pub const TITLE: f32 = 26.0;
pub const SECTION: f32 = 20.0;
pub const CARD: f32 = 15.0;
/// Text: the line under a title (and Simple mode's main field), reading text, secondary text
/// and list rows, fine print (paths, counts, notes), and tags painted in tight spots.
pub const LEAD: f32 = 15.0;
pub const BODY: f32 = 13.5;
pub const SMALL: f32 = 12.5;
pub const CAPTION: f32 = 11.5;
pub const TINY: f32 = 11.0;

/// A heading: Inter semibold, at one of the heading sizes above.
pub fn heading(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(semibold(size))
}

/// `heading` laid out for painting, wrapped at `wrap`.
pub fn heading_galley(painter: &egui::Painter, text: impl Into<String>, size: f32, color: Color32, wrap: f32) -> Arc<egui::Galley> {
    painter.layout(text.into(), semibold(size), color, wrap)
}
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

/// `a` blended toward `b` by `t` (0..=1), channel by channel (alpha included).
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()), m(a.a(), b.a()))
}

/// A short label (captions, buttons, the title bar) in sentence case. `size` is the old
/// uppercase-mono size; mixed-case Inter needs a point more to read the same.
pub fn label_text(text: &str, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(medium(size + 1.0))
}

/// Scroll bars that can always be seen and grabbed. egui's thin preset paints the thumb in the
/// button fill and its track in the field colour, which here are the same, and narrows it to 2 px
/// at rest, so it showed only on hover. The thumb is the text colour, see-through enough to stay
/// quiet but 3:1 against the surfaces it sits on (WCAG 1.4.11), a thin 6 px capsule floating
/// over the content at rest and 9 px under the pointer, with no track until it's grabbed.
pub fn scroll_style() -> egui::style::ScrollStyle {
    egui::style::ScrollStyle {
        floating: true,
        bar_width: 9.0,
        floating_width: 6.0,
        floating_allocated_width: 0.0,
        bar_inner_margin: 3.0,
        bar_outer_margin: 2.0,
        foreground_color: true,
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.0,
        interact_background_opacity: 0.25,
        dormant_handle_opacity: SCROLL_THUMB_AT_REST,
        active_handle_opacity: 0.65,
        interact_handle_opacity: 0.85,
        ..egui::style::ScrollStyle::solid()
    }
}

/// How opaque a scroll bar's thumb is at rest (over the text colour).
const SCROLL_THUMB_AT_REST: f32 = 0.5;

/// `t.text` at `alpha` (0..1): the see-through tints hover, press and grouping use, so they
/// read the same on any surface.
fn text_tint(t: &Tokens, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(t.text.r(), t.text.g(), t.text.b(), (alpha.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// Apply a theme. `roomy` = Beginner mode (bigger type, more air).
pub fn apply(ctx: &egui::Context, kind: ThemeKind, roomy: bool) {
    let t = Tokens::for_kind(kind);
    ctx.data_mut(|d| d.insert_temp(tokens_id(), t));
    ctx.set_theme(if t.dark() { egui::Theme::Dark } else { egui::Theme::Light });
    let mut v = if t.dark() { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.chrome;
    // Dialogs, menus and popups float: a step above the cards in the dark theme (white in the
    // light one), a faint inner outline instead of a border, and a soft wide shadow.
    v.window_fill = if t.dark() { mix(t.card, t.raised, 0.5) } else { t.canvas };
    v.window_stroke = Stroke::new(1.0, text_tint(&t, if t.dark() { 0.12 } else { 0.10 }));
    v.extreme_bg_color = t.field;
    v.faint_bg_color = t.raised;
    v.code_bg_color = t.code_bg;
    v.override_text_color = Some(t.text);
    v.hyperlink_color = t.accent_ink;
    v.warn_fg_color = t.warning;
    v.error_fg_color = t.danger;
    v.window_corner_radius = t.r_xl();
    v.menu_corner_radius = t.r_lg();
    v.window_shadow = egui::Shadow { offset: [0, 12], blur: 60, spread: 0, color: t.shadow };
    v.popup_shadow = egui::Shadow { offset: [0, 8], blur: 40, spread: 0, color: t.shadow };
    v.text_edit_bg_color = Some(t.field);
    // Selected text is a grey wash; a focused field gets a 2 px ring in the secondary grey.
    v.selection.bg_fill = text_tint(&t, 0.20);
    v.selection.stroke = Stroke::new(2.0, t.text_dim);
    v.handle_shape = egui::style::HandleShape::Circle;
    v.striped = false;
    v.indent_has_left_vline = false;
    v.slider_trailing_fill = true;
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = t.card;
    w.noninteractive.weak_bg_fill = t.card;
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.separator);
    w.noninteractive.fg_stroke = Stroke::new(1.0, t.text_dim);
    w.noninteractive.corner_radius = t.r();
    // Buttons and combo boxes are soft grey capsules with no outline: feedback is the tint
    // getting stronger (hover, press), never a border. `bg_fill` is for solid parts (a slider's
    // rail, a check box), in the secondary grey.
    for (wv, weak, bg) in [
        (&mut w.inactive, text_tint(&t, 0.08), t.secondary_fill),
        (&mut w.hovered, text_tint(&t, 0.13), mix(t.secondary_fill, t.text, 0.08)),
        (&mut w.active, text_tint(&t, 0.18), mix(t.secondary_fill, t.text, 0.14)),
        (&mut w.open, text_tint(&t, 0.13), mix(t.secondary_fill, t.text, 0.08)),
    ] {
        wv.bg_fill = bg;
        wv.weak_bg_fill = weak;
        wv.bg_stroke = Stroke::NONE;
        wv.fg_stroke = Stroke::new(1.5, t.text);
        wv.corner_radius = t.r();
        wv.expansion = 0.0;
    }
    ctx.set_visuals(v);
    // Simple mode is for anyone, including people who find small text hard to read: a step up
    // the ramp everywhere and more air.
    let body = if roomy { 15.0 } else { 13.0 };
    ctx.global_style_mut(|s| {
        s.text_styles = [
            (TextStyle::Small, FontId::proportional(if roomy { 12.5 } else { 11.0 })),
            (TextStyle::Body, FontId::proportional(body)),
            (TextStyle::Button, medium(if roomy { 14.0 } else { 13.0 })),
            (TextStyle::Heading, semibold(if roomy { SECTION } else { 17.0 })),
            (TextStyle::Monospace, FontId::monospace(if roomy { 13.5 } else { 12.5 })),
        ]
        .into();
        s.spacing.item_spacing = if roomy { egui::vec2(10.0, 8.0) } else { egui::vec2(8.0, 6.0) };
        s.spacing.button_padding = if roomy { egui::vec2(14.0, 7.0) } else { egui::vec2(12.0, 5.0) };
        s.spacing.interact_size = egui::vec2(24.0, if roomy { 30.0 } else { 26.0 });
        s.spacing.menu_margin = egui::Margin::same(6);
        s.spacing.window_margin = egui::Margin::same(20);
        s.spacing.icon_width = 14.0;
        s.spacing.scroll = scroll_style();
        s.spacing.tooltip_width = 320.0;
        s.interaction.tooltip_delay = 0.3;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG relative luminance of an opaque colour.
    fn luminance(c: Color32) -> f64 {
        let ch = |v: u8| {
            let s = f64::from(v) / 255.0;
            if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
    }

    /// Releases are built with craft-fonts (#19), and so is CI; a build without them has only
    /// Inter, and shows other scripts as boxes.
    #[test]
    fn with_craft_fonts_every_script_people_type_shows() {
        if CRAFT_FONTS.is_empty() {
            return;
        }
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        ctx.run_ui(egui::RawInput::default(), |_| {}).textures_delta.clear();
        let texts = [
            ("Korean", "한국어 사용자"),
            ("Chinese", "你好，世界"),
            ("Japanese", "こんにちは、カタカナ"),
            ("Thai", "ภาษาไทย"),
            ("Arabic", "مرحبا بالعالم"),
            ("Hebrew", "שלום עולם"),
            ("Hindi", "नमस्ते दुनिया"),
            ("Vietnamese", "Tiếng Việt"),
            ("Russian", "Привет"),
            ("Greek", "Γειά σου"),
            ("arrows", "Settings → AI ← ↑ ↓"),
        ];
        for (script, text) in texts {
            for font in [FontId::proportional(14.0), FontId::monospace(14.0), medium(14.0), semibold(14.0)] {
                assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, text)), "{script} ({text}) shows as boxes in {font:?}");
            }
        }
    }

    #[test]
    fn a_scroll_thumb_at_rest_stands_out_from_every_surface_in_every_theme() {
        let style = scroll_style();
        assert!(style.floating_width >= 6.0 && style.foreground_color, "visible and wide enough to grab at rest");
        for k in ThemeKind::ALL {
            let t = Tokens::for_kind(k);
            for (name, surface) in [("chrome", t.chrome), ("canvas", t.canvas), ("card", t.card)] {
                // The thumb is the text colour at that opacity, over the surface.
                let a = f64::from(style.dormant_handle_opacity);
                let mix = |f: u8, b: u8| (a * f64::from(f) + (1.0 - a) * f64::from(b)).round() as u8;
                let thumb = Color32::from_rgb(mix(t.text.r(), surface.r()), mix(t.text.g(), surface.g()), mix(t.text.b(), surface.b()));
                let (l1, l2) = (luminance(thumb), luminance(surface));
                let ratio = (l1.max(l2) + 0.05) / (l1.min(l2) + 0.05);
                assert!(ratio >= 3.0, "{k:?} {name}: the thumb at rest is only {ratio:.2}:1");
            }
        }
    }

    #[test]
    fn theme_ids_round_trip() {
        for k in ThemeKind::ALL {
            assert_eq!(ThemeKind::from_id(k.id()), k);
        }
        assert_eq!(ThemeKind::from_id("nonsense"), ThemeKind::Studio);
    }

    /// WCAG 2.x contrast ratio between two opaque colours.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let lum = |c: Color32| {
            let ch = |v: u8| {
                let s = v as f32 / 255.0;
                if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
        };
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn contrast_meets_wcag_aa_in_both_themes() {
        for k in ThemeKind::ALL {
            let t = Tokens::for_kind(k);
            // `success` is text too: the Clean room and Ready pills.
            for (name, fg) in [("text", t.text), ("text_dim", t.text_dim), ("text_faint", t.text_faint), ("accent_ink", t.accent_ink), ("success", t.success)] {
                for bg in [t.canvas, t.card, t.chrome, t.raised] {
                    assert!(contrast(fg, bg) >= 4.5, "{k:?} {name} on {bg:?}: {:.2}", contrast(fg, bg));
                }
            }
            assert!(contrast(t.on_accent, t.accent) >= 4.5, "{k:?} text on the app colour: {:.2}", contrast(t.on_accent, t.accent));
            assert!(contrast(t.field_border, t.canvas) >= 3.0, "{k:?} control borders: {:.2}", contrast(t.field_border, t.canvas));
            // Main buttons are ink-filled with the canvas colour as their label.
            assert!(contrast(t.canvas, t.text) >= 4.5, "{k:?} main buttons");
        }
    }

    #[test]
    fn confidence_has_four_distinct_colours() {
        let cs = [
            confidence_color(Some(Confidence::Observed)),
            confidence_color(Some(Confidence::Derived)),
            confidence_color(Some(Confidence::Inferred)),
            confidence_color(None),
        ];
        for (i, a) in cs.iter().enumerate() {
            for b in cs.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        assert_eq!(confidence_label(None), "Unknown");
    }
}
