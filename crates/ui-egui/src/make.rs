//! Simple mode's front door, for people new to computers: two numbered questions in plain words
//! ("What do you want to recreate?", "Say what you want"), big targets and one big Start button. Below it,
//! the projects made so far.

use crate::i18n::tr;
use crate::state::MakeDraft;
use crate::theme::{self, Tokens};
use crate::{CraftApp, Pick, icons, widgets};
use craft_anything_engine::workspace::{Status, TargetRef};
use egui::{Color32, Margin, Response, RichText, Sense, Stroke, Ui, Vec2};

const SUGGESTIONS_WEB: &[&str] = &["Recreate this website", "Recreate it with darker colours"];
const SUGGESTIONS_APP: &[&str] = &["Make my own version of this app", "Recreate just its main feature"];
const SUGGESTIONS_GAME: &[&str] = &["Make a game that plays like this", "Make a simpler version of this game"];
const SUGGESTIONS_NONE: &[&str] = &["A website for my bakery", "A simple snake game"];

/// The first http(s) address in some text.
pub fn find_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| matches!(c, '"' | '\'' | '(' | ')' | '<' | '>' | ',' | '.' | ';')))
        .find(|w| w.starts_with("http://") || w.starts_with("https://"))
        .map(str::to_string)
}

/// Why a typed web address can't be used, or `None` when it's fine (or empty).
pub fn url_problem(typed: &str) -> Option<&'static str> {
    let t = typed.trim();
    if t.is_empty() {
        return None;
    }
    let url = normalize_url(t);
    let host = url.split("://").nth(1).unwrap_or_default().split(['/', '?', '#']).next().unwrap_or_default();
    let host = host.rsplit('@').next().unwrap_or_default().split(':').next().unwrap_or_default();
    let ok = (url.starts_with("http://") || url.starts_with("https://"))
        && !t.chars().any(char::is_whitespace)
        && (host == "localhost" || (host.contains('.') && !host.starts_with('.') && !host.ends_with('.')));
    (!ok).then_some("Enter a full web address, like example.com or https://example.com/page.")
}

/// "example.com" → "https://example.com"; anything with a scheme is kept.
pub fn normalize_url(s: &str) -> String {
    let t = s.trim();
    if t.is_empty() || t.contains("://") { t.to_string() } else { format!("https://{t}") }
}

/// The target the draft will use: the picked one, the typed address, or an address in the request.
pub fn resolve_target(d: &MakeDraft) -> TargetRef {
    if d.target != TargetRef::None {
        return d.target.clone();
    }
    if !d.url.trim().is_empty() {
        return TargetRef::parse(&normalize_url(&d.url));
    }
    find_url(&d.request).map(|url| TargetRef::Website { url }).unwrap_or(TargetRef::None)
}

/// A game is being copied or made, so it's built with a game engine ("Build with").
pub fn is_game(kinds: &craft_anything_ai::rebuild::KindRegistry, d: &MakeDraft) -> bool {
    use craft_anything_ai::rebuild::GameKind;
    d.found.is_some()
        || (d.game && matches!(d.target, TargetRef::Local { .. }))
        || kinds.choose(&d.request, &resolve_target(d), false).is_some_and(|(k, _)| k.id() == GameKind::ID)
}

/// The front door: make your own version of something, or just look inside it. A calm, centred
/// start like a fresh conversation: the headline, one quiet line, then the choices as pills and
/// the request in the floating prompt box.
pub fn hero(app: &mut CraftApp, ui: &mut Ui, w: f32) {
    ui.vertical_centered(|ui| {
        if let Some(i) = widgets::segmented_sized(ui, &["Make my own version", "Just look inside"], usize::from(app.state.make.look), 34.0, 12.5) {
            app.state.make.look = i == 1;
        }
    });
    ui.add_space(28.0);
    if app.state.make.look {
        look(app, ui, w);
        return;
    }
    headline(ui, "What do you want to make?", "Choose what to recreate, say what you want, then press Start.");
    ui.add_space(26.0);
    section(ui, "What do you want to recreate?");
    ui.add_space(10.0);
    choose_target(app, ui);
    // How closely the original may be looked at, once there is one.
    let d = &app.state.make;
    if matches!(d.target, TargetRef::Local { .. }) || d.editing_url || !d.url.is_empty() {
        ui.add_space(18.0);
        look_inside(app, ui);
    }
    ui.add_space(26.0);
    section(ui, "Say what you want");
    ui.add_space(10.0);
    // The example matches the choice above (a "recreate this website" hint under
    // "Nothing — start fresh" contradicts the choice).
    let d = &app.state.make;
    let hint = match &d.target {
        TargetRef::Local { .. } if d.game => "For example: the same game, with a high-score table",
        TargetRef::Local { .. } => "For example: the same app, but simpler",
        TargetRef::Website { .. } => "For example: recreate this website",
        TargetRef::None if d.editing_url || !d.url.is_empty() => "For example: recreate this website",
        TargetRef::None => "For example: a website for my bakery",
    };
    // Websites are photographed for the builder; anything else looks right from pictures.
    let website = matches!(resolve_target(d), TargetRef::Website { .. }) || d.editing_url;
    let out = crate::prompt_box::show(
        ui,
        &mut app.state.make.request,
        &app.state.make.pictures,
        crate::prompt_box::PromptBox {
            name: "Say what you want",
            hint: tr(hint),
            rows: (3, 10),
            attach: (!website).then_some("Add pictures of the original (optional: screenshots help it look right)"),
            stop_button: None,
            send: false,
            recall: false,
            history: None,
            font_size: theme::LEAD,
        },
    );
    if let Some(i) = out.remove {
        app.state.make.pictures.remove(i);
    }
    if out.attach {
        app.pick(Pick::Pictures);
    }
    let submit = out.send;
    if app.state.make.request.trim().is_empty() {
        ui.add_space(12.0);
        suggestions(app, ui);
    }
    if is_game(&app.kinds, &app.state.make) {
        ui.add_space(24.0);
        crate::engine_ui::build_with(app, ui);
        ui.add_space(14.0);
        let on = app.state.make.generate_assets;
        crate::engine_ui::assets_toggle(app, ui, on);
    }
    ui.add_space(22.0);
    ui.horizontal(|ui| {
        crate::builders::built_by(app, ui);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let ready = !app.state.make.request.trim().is_empty() && url_problem(&app.state.make.url).is_none() && app.creating.is_none();
            let start = widgets::primary_sized(ui, Some("sparkles"), "Start", ready, 40.0, 13.5);
            let start = if ready { start } else { start.on_disabled_hover_text(tr("Say what you want first.")) };
            if (start.clicked() || submit) && ready {
                app.start_make();
            }
        });
    });
    // Where it's saved: secondary to Start, so under it.
    ui.add_space(14.0);
    crate::folders::make_line(app, ui);
}

/// "Just look inside": what to look at, each opening straight away. Nothing is built or run.
fn look(app: &mut CraftApp, ui: &mut Ui, w: f32) {
    headline(ui, "What do you want to look inside?", "We only read it; nothing runs. Every answer shows how sure we are.");
    ui.add_space(30.0);
    // Each of these reads the original, which Clean room never does.
    if !app.settings.deep_mode {
        soft_card(&Tokens::get(ui.ctx())).show(ui, |ui| {
            ui.set_width(w - 40.0);
            crate::engine_ui::deep_needed(app, ui);
        });
        return;
    }
    ui.spacing_mut().item_spacing.y = 8.0;
    if option_card(ui, w, "app-window", "An app or a file", "See how a program works: what it does, what it talks to, what's inside.", false).clicked() {
        app.pick(Pick::OpenFile);
    }
    if option_card(ui, w, "globe", "A website", "See how a page is built: its layout, styles, scripts and the requests it makes.", false).clicked() {
        app.state.modal = Some(crate::state::Modal::Website);
    }
    if option_card(ui, w, "package", "A game's files", "See what a game packs, and save its pictures as PNG files.", false).clicked() {
        app.state.screen = crate::state::Screen::GameFiles;
    }
}

/// An empty state's centred headline and the quiet line under it.
pub(crate) fn headline(ui: &mut Ui, title: &str, sub: &str) {
    let t = Tokens::get(ui.ctx());
    ui.vertical_centered(|ui| {
        ui.label(theme::heading(tr(title), HEADLINE));
        if !sub.is_empty() {
            ui.add_space(8.0);
            ui.label(RichText::new(tr(sub)).size(theme::SMALL).color(t.text_dim));
        }
    });
}

/// The size of a screen's centred headline.
pub(crate) const HEADLINE: f32 = 22.0;

/// A quiet section label above a group of choices.
pub(crate) fn section(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.label(RichText::new(tr(text)).font(theme::medium(theme::SMALL)).color(t.text_dim));
}

/// The primary text colour at `a` (0..=1) opacity: the tints that step a surface up (groups,
/// hover, the chosen one). Works on the page and inside a sheet alike.
pub(crate) fn tint(t: &Tokens, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(t.text.r(), t.text.g(), t.text.b(), (a.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// A soft card: a borderless r16 surface a step above what's behind it.
pub(crate) fn soft_card(t: &Tokens) -> egui::Frame {
    egui::Frame::new().fill(tint(t, CARD_TINT)).corner_radius(t.r_lg()).inner_margin(Margin::same(20))
}

/// How far a card steps up from what's behind it (and a hovered one).
pub(crate) const CARD_TINT: f32 = 0.045;
pub(crate) const CARD_HOVER_TINT: f32 = 0.08;

/// A keyboard-focus ring that follows a pill's round ends.
pub(crate) fn pill_focus(ui: &Ui, resp: &Response, rect: egui::Rect) {
    if resp.has_focus() {
        let t = Tokens::get(ui.ctx());
        ui.painter().rect_stroke(rect.expand(2.0), t.r_pill(), Stroke::new(2.0, t.text_dim), egui::StrokeKind::Outside);
    }
}

/// A choice as a pill: outlined and quiet at rest, filled when chosen (the fill, not colour alone,
/// says which). Radio-like for screen readers.
pub(crate) fn chip(ui: &mut Ui, icon: Option<&str>, label: &str, on: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let text = tr(label);
    let g = ui.painter().layout_no_wrap(text.clone(), theme::medium(theme::SMALL), t.text);
    let icon_w = if icon.is_some() { 22.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(g.size().x + icon_w + 32.0, 36.0), Sense::click());
    let h = widgets::hover_t(ui, &resp);
    let fill = if on { tint(&t, 0.15) } else { tint(&t, 0.06 * h) };
    ui.painter().rect_filled(rect, t.r_pill(), fill);
    if !on {
        ui.painter().rect_stroke(rect, t.r_pill(), Stroke::new(1.0, t.chrome_stroke), egui::StrokeKind::Inside);
    }
    let fg = if on { t.text } else { theme::mix(t.text_dim, t.text, h) };
    let mut x = rect.left() + 16.0;
    if let Some(i) = icon {
        icons::paint(ui, egui::Rect::from_center_size(egui::pos2(x + 7.5, rect.center().y), Vec2::splat(15.0)), i, 15.0, fg);
        x += icon_w;
    }
    ui.painter().galley_with_override_text_color(egui::pos2(x, rect.center().y - g.size().y / 2.0), g, fg);
    pill_focus(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, on, &text));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A bigger choice: a borderless r16 card with an icon disc, a title and a line under it. The
/// chosen one is filled a step brighter, its disc turns white (black in light) and a tick shows.
pub(crate) fn option_card(ui: &mut Ui, w: f32, icon: &str, title: &str, blurb: &str, on: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let wrap = (w - 104.0).max(60.0);
    let g = ui.painter().layout(tr(title), theme::medium(theme::BODY), t.text, wrap);
    let sub = (!blurb.is_empty()).then(|| ui.painter().layout(tr(blurb), egui::FontId::proportional(theme::SMALL), t.text_dim, wrap));
    let text_h = g.size().y + sub.as_ref().map_or(0.0, |s| s.size().y + 4.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, (text_h + 32.0).max(68.0)), Sense::click());
    let h = widgets::hover_t(ui, &resp);
    let fill = if on { tint(&t, 0.11) } else { tint(&t, CARD_TINT + (CARD_HOVER_TINT - CARD_TINT) * h) };
    ui.painter().rect_filled(rect, t.r_lg(), fill);
    let disc = egui::Rect::from_center_size(egui::pos2(rect.left() + 34.0, rect.center().y), Vec2::splat(36.0));
    ui.painter().circle_filled(disc.center(), 18.0, if on { t.accent } else { tint(&t, 0.08) });
    icons::paint(ui, disc, icon, 18.0, if on { t.on_accent } else { t.text });
    let top = egui::pos2(disc.right() + 14.0, rect.center().y - text_h / 2.0);
    let title_h = g.size().y;
    ui.painter().galley(top, g, t.text);
    if let Some(s) = sub {
        ui.painter().galley(top + Vec2::new(0.0, title_h + 4.0), s, t.text_dim);
    }
    if on {
        icons::paint(ui, egui::Rect::from_center_size(egui::pos2(rect.right() - 24.0, rect.center().y), Vec2::splat(18.0)), "check", 18.0, t.text);
    }
    if resp.has_focus() {
        ui.painter().rect_stroke(rect.expand(2.0), t.radius_lg + 2.0, Stroke::new(2.0, t.text_dim), egui::StrokeKind::Outside);
    }
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, on, tr(title)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A small, quiet text button (secondary text, brightening with an underline on hover).
pub(crate) fn quiet_link(ui: &mut Ui, label: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let r = ui.add(egui::Label::new(RichText::new(tr(label)).size(theme::SMALL).color(t.text_dim)).sense(Sense::click()));
    if r.hovered() || r.has_focus() {
        ui.painter().text(r.rect.left_top(), egui::Align2::LEFT_TOP, tr(label), egui::FontId::proportional(theme::SMALL), t.text);
        ui.painter().hline(r.rect.x_range(), r.rect.bottom(), Stroke::new(1.0, t.text_dim));
    }
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Make the combo boxes inside `ui` quiet pills: no fill at rest, a faint ring, a tint on hover.
pub(crate) fn pill_combos(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let w = &mut ui.visuals_mut().widgets;
    for (v, fill, ring) in [
        (&mut w.inactive, Color32::TRANSPARENT, t.chrome_stroke),
        (&mut w.hovered, tint(&t, 0.06), t.chrome_stroke),
        (&mut w.active, tint(&t, 0.10), t.chrome_stroke),
        (&mut w.open, tint(&t, 0.06), t.chrome_stroke),
    ] {
        v.weak_bg_fill = fill;
        v.bg_fill = fill;
        v.bg_stroke = Stroke::new(1.0, ring);
        v.corner_radius = t.r_pill();
    }
    ui.spacing_mut().button_padding = Vec2::new(14.0, 7.0);
}

/// An idea that fills in the request: a borderless r16 card that brightens on hover.
fn idea(ui: &mut Ui, text: &str, w: f32) -> Response {
    let t = Tokens::get(ui.ctx());
    let g = ui.painter().layout(tr(text), egui::FontId::proportional(theme::SMALL), t.text, w - 52.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, (g.size().y + 26.0).max(48.0)), Sense::click());
    let h = widgets::hover_t(ui, &resp);
    ui.painter().rect_filled(rect, t.r_lg(), tint(&t, CARD_TINT + (CARD_HOVER_TINT - CARD_TINT) * h));
    let fg = theme::mix(t.text_dim, t.text, h);
    icons::paint(ui, egui::Rect::from_center_size(egui::pos2(rect.left() + 24.0, rect.center().y), Vec2::splat(15.0)), "sparkles", 14.0, t.text_faint);
    ui.painter().galley_with_override_text_color(egui::pos2(rect.left() + 42.0, rect.center().y - g.size().y / 2.0), g, fg);
    if resp.has_focus() {
        ui.painter().rect_stroke(rect.expand(2.0), t.radius_lg + 2.0, Stroke::new(2.0, t.text_dim), egui::StrokeKind::Outside);
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(text)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn choose_target(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let d = &app.state.make;
    let local = matches!(d.target, TargetRef::Local { .. }).then_some(d.game);
    let website = local.is_none() && (d.editing_url || !d.url.is_empty());
    let scratch = local.is_none() && !website;
    let mut pick = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(8.0);
        for (i, (icon, label, on)) in [
            ("globe", "A website", website),
            ("app-window", "An app", local == Some(false)),
            ("puzzle", "A game", local == Some(true)),
            ("sparkles", "Nothing — start fresh", scratch),
        ]
        .into_iter()
        .enumerate()
        {
            if chip(ui, Some(icon), label, on).clicked() {
                pick = Some(i);
            }
        }
    });
    let m = &mut app.state.make;
    if matches!(pick, Some(0) | Some(3)) {
        // A different kind of start: forget the last game's engine and the engine picked for it.
        m.found = None;
        m.builder = None;
    }
    match pick {
        Some(0) => {
            m.target = TargetRef::None;
            m.editing_url = true;
        }
        Some(1) => app.pick(Pick::TargetFile),
        Some(2) => app.pick(Pick::TargetFolder),
        Some(_) => {
            m.target = TargetRef::None;
            m.url.clear();
            m.editing_url = false;
        }
        None => {}
    }
    let m = &mut app.state.make;
    // Recomputed so the address field shows on the very frame "A website" is clicked.
    let website = !matches!(m.target, TargetRef::Local { .. }) && (m.editing_url || !m.url.is_empty());
    if let TargetRef::Local { path } = m.target.clone() {
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("You chose:")).size(theme::SMALL).color(t.text_dim));
            ui.label(RichText::new(m.target.label()).font(theme::medium(theme::SMALL)));
            ui.label(RichText::new(crate::short_path(&path)).color(t.text_faint).size(theme::CAPTION));
        });
    } else if website {
        ui.add_space(16.0);
        ui.label(RichText::new(tr("Type the website's address:")).size(theme::SMALL).color(t.text_dim));
        ui.add_space(6.0);
        let r = ui.add(
            egui::TextEdit::singleline(&mut m.url)
                .hint_text("example.com")
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(theme::BODY))
                .frame(widgets::text_field_frame(&t).corner_radius(t.r()).inner_margin(Margin::symmetric(14, 10))),
        );
        if pick == Some(0) {
            r.request_focus();
        }
        // Say what's wrong right under the field, once something has been typed.
        if let Some(problem) = url_problem(&m.url) {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                icons::show(ui, "triangle-alert", 16.0, t.danger);
                ui.label(RichText::new(tr(problem)).color(t.danger).size(theme::SMALL));
            });
        }
    }
}

/// Clean room or Deep for what is recreated: the title bar's switch, as a plain choice.
fn look_inside(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // The app's one setting (the title bar's switch), not this project's: said where it's chosen.
    ui.horizontal_wrapped(|ui| {
        section(ui, "How closely may Craft Anything look at it?");
        ui.label(RichText::new(tr("For everything you make, like the switch in the title bar.")).size(theme::CAPTION).color(t.text_faint));
    });
    ui.add_space(10.0);
    let deep = app.settings.deep_mode;
    let choices = [
        (false, "shield-check", "Clean room", "Nothing inside it is read; Craft Anything works from what you can see. Your version gets its own art."),
        (true, "microscope", "Deep", "Craft Anything reads its code and files with every tool it has. For your private use."),
    ];
    let mut pick = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for (on, icon, label, _) in choices {
            if chip(ui, Some(icon), label, deep == on).clicked() {
                pick = Some(on);
            }
        }
    });
    // What the chosen one means, in a quiet line under the pills.
    if let Some((_, _, _, blurb)) = choices.iter().find(|c| c.0 == deep) {
        ui.add_space(8.0);
        ui.add(egui::Label::new(RichText::new(tr(blurb)).size(theme::CAPTION + 1.0).color(t.text_dim)).wrap());
    }
    if let Some(on) = pick {
        app.set_deep(on);
    }
    // Ghidra reads programs, not websites.
    if !matches!(app.state.make.target, TargetRef::Local { .. }) {
        return;
    }
    if app.settings.deep_mode && !app.ghidra_missing() {
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            icons::show(ui, "cpu", 16.0, t.icon);
            let line = format!("{} {}", crate::engine_ui::ghidra_memory_words(app), tr("More makes a big program's first look faster."));
            ui.label(RichText::new(line).size(theme::SMALL).color(t.text_dim)).on_hover_text(crate::engine_ui::ghidra_memory_guide());
            if quiet_link(ui, "Change").clicked() {
                app.state.settings_open = true;
                app.state.settings_tab = crate::state::SettingsTab::Engines;
            }
        });
    }
    if app.settings.deep_mode && app.ghidra_missing() {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            icons::show(ui, "triangle-alert", 16.0, t.warning);
            ui.label(RichText::new(tr("Ghidra isn't on this computer yet, so the built-in decompiler stands in.")).size(theme::SMALL).color(t.text_dim));
            if app.ghidra.job.is_none() && widgets::secondary(ui, Some("download"), "Get Ghidra").clicked() {
                app.state.modal = Some(crate::state::Modal::Ghidra);
            }
        });
    }
}

fn suggestions(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let list = match resolve_target(&app.state.make) {
        TargetRef::Website { .. } => SUGGESTIONS_WEB,
        TargetRef::Local { .. } if app.state.make.game => SUGGESTIONS_GAME,
        TargetRef::Local { .. } => SUGGESTIONS_APP,
        TargetRef::None => SUGGESTIONS_NONE,
    };
    ui.label(RichText::new(tr("Ideas:")).size(theme::CAPTION).color(t.text_faint));
    ui.add_space(6.0);
    let gap = 8.0;
    let w = ((ui.available_width() - gap) / 2.0).floor();
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(gap, gap);
        for s in list.iter().take(2) {
            if idea(ui, s, w).clicked() {
                app.state.make.request = tr(s);
            }
        }
    });
}

/// Time since `ms` in plain words.
pub fn ago(ms: u64) -> String {
    let now = craft_anything_engine::inbox::now_ms();
    let s = now.saturating_sub(ms) / 1000;
    match s {
        0..=59 => tr("just now"),
        60..=3599 => crate::i18n::trf("{n} min ago", &[("n", &(s / 60).to_string())]),
        3600..=86_399 => crate::i18n::trf("{n} h ago", &[("n", &(s / 3600).to_string())]),
        86_400..=172_799 => tr("yesterday"),
        _ => crate::i18n::trf("{n} days ago", &[("n", &(s / 86_400).to_string())]),
    }
}

/// A project's status as the app shows it: waiting its turn, interrupted (its run stopped
/// part-way: the app or the computer closed), or the one it saved. (label, text, fill).
pub fn status_shown(app: &CraftApp, ws: &craft_anything_engine::workspace::Workspace, t: &Tokens) -> (&'static str, egui::Color32, egui::Color32) {
    let s = ws.manifest.status;
    let (label, look) = if app.asking.contains(&ws.root) {
        ("Needs you", Status::NeedsYou)
    } else if app.runs.position(&ws.root).is_some() {
        ("Waiting", s)
    } else if crate::project::interrupted(app, ws) {
        ("Interrupted", Status::NeedsYou)
    } else {
        (s.label(), s)
    };
    let (fg, bg) = status_colors(t, look);
    (label, fg, bg)
}

pub fn status_colors(t: &Tokens, s: Status) -> (egui::Color32, egui::Color32) {
    match s {
        Status::Ready => (t.success, t.raised),
        Status::NeedsYou => (t.warning, t.raised),
        Status::Stopped | Status::New => (t.text_dim, t.raised),
        _ => (t.accent_ink, t.accent_soft),
    }
}

/// A project's icon: a game (built with a game engine), a website, an app, or something new.
pub fn project_icon(m: &craft_anything_engine::workspace::Manifest) -> &'static str {
    if craft_anything_engines::game::Builder::from_stack(&m.stack).is_some() {
        return "puzzle";
    }
    match m.target {
        TargetRef::Website { .. } => "globe",
        TargetRef::Local { .. } => "app-window",
        TargetRef::None => "sparkles",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_addresses_are_checked_inline() {
        for ok in ["", "example.com", "https://a.b/c?d", "http://localhost:5173", "sub.example.co.uk/page"] {
            assert!(url_problem(ok).is_none(), "{ok}");
        }
        for bad in ["example", "https://", "two words.com", "ftp://x.y", "https://.com", "x."] {
            assert!(url_problem(bad).is_some(), "{bad}");
        }
    }

    #[test]
    fn targets_from_the_draft() {
        let mut d = MakeDraft { request: "Please recreate https://example.com/pricing.".into(), ..Default::default() };
        assert_eq!(resolve_target(&d), TargetRef::Website { url: "https://example.com/pricing".into() });
        d.url = "acme.io".into();
        assert_eq!(resolve_target(&d), TargetRef::Website { url: "https://acme.io".into() }, "the typed address wins");
        d.url.clear();
        d.request = "a snake game".into();
        assert_eq!(resolve_target(&d), TargetRef::None);
        assert_eq!(normalize_url("http://x.y"), "http://x.y");
    }
}
