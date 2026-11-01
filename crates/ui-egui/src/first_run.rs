//! First run: Welcome → Who builds (and answers) → Purpose → Done. Four short screens, all
//! skippable. Making something needs a builder, so that's what the AI step sets up first.

use crate::i18n::tr;
use crate::state::FirstRunStep;
use crate::theme::{self, Tokens};
use crate::{CraftApp, icons, widgets};
use craft_anything_ai::{BackendKind, Detected};
use craft_anything_engine::Mode;
use egui::{RichText, Ui, Vec2};

pub fn show(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let full = ui.max_rect();
    let width = 560.0_f32.min(full.width() - 48.0);
    ui.add_space(((full.height() - 560.0) / 2.0).clamp(24.0, 120.0));
    ui.vertical_centered(|ui| {
        ui.set_max_width(width);
        match app.state.first_run {
            FirstRunStep::Welcome => welcome(app, ui),
            FirstRunStep::Brain => brain(app, ui, width),
            FirstRunStep::Purpose => purpose(app, ui, width),
            FirstRunStep::Done => done(app, ui),
        }
    });
    // Where you are: four small pills (the done ones bright) over "02 / 04  AI".
    let step = app.state.first_run.index();
    let y = full.bottom() - 30.0;
    let names = ["Welcome", "AI", "Purpose", "Ready"];
    let label = format!("{:02} / 04   {}", step + 1, tr(names.get(step).copied().unwrap_or_default()));
    let g = ui.painter().layout_no_wrap(label, egui::FontId::proportional(theme::TINY + 0.5), t.text_faint);
    ui.painter().galley_with_override_text_color(egui::pos2(full.center().x - g.size().x / 2.0, y - g.size().y / 2.0), g, t.text_faint);
    let dots_w = 4.0 * 18.0 + 3.0 * 6.0;
    for i in 0..4 {
        let x = full.center().x - dots_w / 2.0 + i as f32 * 24.0;
        let r = egui::Rect::from_min_size(egui::pos2(x, y - 20.0), Vec2::new(18.0, 4.0));
        ui.painter().rect_filled(r, t.r_pill(), if i <= step { t.text } else { crate::make::tint(&t, 0.15) });
    }
    if step > 0 && step < 3 {
        let g = ui.painter().layout_no_wrap(tr("← Back"), theme::medium(theme::SMALL), t.text);
        let r = egui::Rect::from_min_size(egui::pos2(full.left() + 20.0, y - 18.0), Vec2::new(g.size().x + 28.0, 32.0));
        let resp = ui.interact(r, ui.id().with("fr-back"), egui::Sense::click());
        let h = widgets::hover_t(ui, &resp);
        ui.painter().rect_filled(r, t.r_pill(), crate::make::tint(&t, 0.06 * h));
        let col = theme::mix(t.text_dim, t.text, h);
        ui.painter().galley_with_override_text_color(egui::pos2(r.left() + 14.0, r.center().y - g.size().y / 2.0), g, col);
        crate::make::pill_focus(ui, &resp, r);
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Back")));
        if resp.clicked() {
            app.state.first_run = match app.state.first_run {
                FirstRunStep::Purpose => FirstRunStep::Brain,
                _ => FirstRunStep::Welcome,
            };
        }
    }
}

fn welcome(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.add(crate::app_icon(84.0));
    ui.add_space(22.0);
    ui.label(theme::heading(tr("Craft Anything"), crate::make::HEADLINE));
    ui.add_space(8.0);
    ui.label(RichText::new(tr("Find out how any software works, then craft your own.")).size(theme::SMALL).color(t.text_dim));
    ui.add_space(36.0);
    let w = 440.0_f32.min(ui.available_width());
    ui.allocate_ui_with_layout(Vec2::new(w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        if crate::make::option_card(ui, w, "graduation-cap", "I'm new to this", "Plain words, big buttons, and I'll explain everything.", false).clicked() {
            app.settings.mode = Mode::Beginner;
            app.state.first_run = FirstRunStep::Brain;
        }
        if crate::make::option_card(
            ui,
            w,
            "terminal",
            "Show me everything",
            "Every tool as well: a workbench with functions, bytes, references and evidence.",
            false,
        )
        .clicked()
        {
            app.settings.mode = Mode::Experienced;
            app.state.first_run = FirstRunStep::Brain;
        }
    });
    ui.add_space(22.0);
    if crate::make::quiet_link(ui, "Skip setup").on_hover_text(tr("Start with Simple mode; you can set up AI later in Settings.")).clicked() {
        app.finish_first_run();
    }
}

fn header(ui: &mut Ui, title: &str, sub: &str) {
    let t = Tokens::get(ui.ctx());
    ui.label(theme::heading(tr(title), crate::make::HEADLINE));
    ui.add_space(8.0);
    ui.label(RichText::new(tr(sub)).size(theme::SMALL).color(t.text_dim));
    ui.add_space(26.0);
}

fn brain(app: &mut CraftApp, ui: &mut Ui, width: f32) {
    let t = Tokens::get(ui.ctx());
    header(
        ui,
        "Who builds what you make",
        "A coding agent such as Claude Code builds it, and an AI answers your questions in plain words. Change this any time in Settings → AI.",
    );
    // Whether something can build yet, in one line: the same rule Make and Start use.
    match app.default_builder() {
        Some(b) => {
            ui.horizontal(|ui| {
                icons::show(ui, "check", 16.0, t.success);
                ui.label(RichText::new(crate::i18n::trf("{name} builds what you make.", &[("name", &b)])).color(t.text_dim));
            });
        }
        None => {
            ui.horizontal_wrapped(|ui| {
                icons::show(ui, "triangle-alert", 16.0, t.warning);
                ui.label(
                    RichText::new(tr("Nothing can build yet: install Claude Code (and sign in to it once), or add an AI with an API key below."))
                        .color(t.text_dim),
                );
            });
        }
    }
    ui.add_space(10.0);
    let detected: Vec<Detected> = app.detected.clone().unwrap_or_default();
    let list_h = (ui.ctx().content_rect().height() - 330.0).clamp(220.0, 520.0);
    egui::ScrollArea::vertical().max_height(list_h).show(ui, |ui| {
        ui.set_width(width);
        if app.detecting {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().color(t.accent));
                ui.label(RichText::new(tr("Looking for AI on this computer…")).color(t.text_dim));
            });
            ui.add_space(8.0);
        }
        for d in &detected {
            brain_card(app, ui, d);
            ui.add_space(8.0);
        }
        if !app.detecting && detected.is_empty() {
            crate::make::soft_card(&t).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    icons::show(ui, "radar", 22.0, t.text_faint);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(tr("We didn't find an AI on this computer yet")).font(theme::medium(theme::BODY)));
                        ui.label(RichText::new(tr("You can add a Claude, ChatGPT or OpenRouter key, a local model (Ollama, LM Studio) or an agent you already use, later in Settings → AI.")).color(t.text_dim));
                    });
                });
            });
            ui.add_space(8.0);
        }
        // "No AI for now"
        let none_on = app.state.backend.is_none();
        let r = option_row(ui, "circle-stop", "No AI for now", "The built-in engines still show you everything they find.", none_on);
        if r.clicked() {
            app.choose_backend(None);
            app.state.first_run = FirstRunStep::Purpose;
        }
    });
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add_space(((ui.available_width() - 240.0) / 2.0).max(0.0));
        if widgets::secondary(ui, Some("refresh-cw"), "Look again").clicked() {
            app.detect_backends();
        }
        if widgets::primary(ui, "Continue").clicked() {
            app.state.first_run = FirstRunStep::Purpose;
        }
    });
}

fn kind_icon(k: BackendKind) -> &'static str {
    match k {
        BackendKind::Anthropic | BackendKind::Openai | BackendKind::Openrouter => "key-round",
        BackendKind::OpenaiCompatible => "server",
        BackendKind::AgentCli => "terminal",
    }
}

fn brain_card(app: &mut CraftApp, ui: &mut Ui, d: &Detected) {
    let t = Tokens::get(ui.ctx());
    let name = d.suggestion.name.clone();
    let on = app.state.backend.as_ref() == Some(&name);
    let configured = app.backends.iter().any(|b| b.name == name);
    let frame = egui::Frame::new()
        .fill(crate::make::tint(&t, if on { 0.11 } else { crate::make::CARD_TINT }))
        .corner_radius(t.r_lg())
        .inner_margin(egui::Margin::symmetric(16, 14));
    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 18.0, if on { t.accent } else { crate::make::tint(&t, 0.08) });
            icons::paint(ui, rect, kind_icon(d.suggestion.kind), 18.0, if on { t.on_accent } else { t.text });
            ui.vertical(|ui| {
                ui.label(RichText::new(&name).font(theme::medium(theme::BODY)));
                ui.label(RichText::new(&d.reason).color(t.text_dim).size(theme::SMALL));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if on {
                    widgets::pill(ui, "Selected", t.on_accent, t.accent);
                } else if configured {
                    if widgets::secondary(ui, None, "Use this").clicked() {
                        app.choose_backend(Some(name.clone()));
                    }
                } else if !d.needs_key && widgets::primary(ui, "Use this").clicked() {
                    app.add_backend(d.suggestion.clone(), None);
                }
            });
        });
        if d.needs_key && !configured {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let key = app.key_inputs.entry(name.clone()).or_default();
                ui.add(
                    egui::TextEdit::singleline(key)
                        .password(true)
                        .hint_text(tr("Paste your API key (kept in your system keychain)"))
                        .desired_width(ui.available_width() - 140.0)
                        .frame(widgets::text_field_frame(&t).corner_radius(t.r())),
                );
                let key = key.trim().to_string();
                if widgets::primary_icon(ui, None, "Connect", !key.is_empty()).clicked() && !key.is_empty() {
                    // The key goes to the keychain; the app keeps no copy of it.
                    app.key_inputs.remove(&name);
                    app.add_backend(d.suggestion.clone(), Some(key));
                }
            });
        }
    });
}

fn option_row(ui: &mut Ui, icon: &str, title: &str, sub: &str, on: bool) -> egui::Response {
    let w = ui.available_width();
    crate::make::option_card(ui, w, icon, title, sub, on)
}

fn purpose(app: &mut CraftApp, ui: &mut Ui, width: f32) {
    let t = Tokens::get(ui.ctx());
    header(ui, "What brings you here?", "One line, in plain words. It stays on this computer and is never sent anywhere.");
    ui.allocate_ui(Vec2::new(width, 400.0), |ui| {
        for (p, title, sub) in crate::state::purposes() {
            let on = app.state.purpose == p;
            if option_row(ui, purpose_icon(p), title, sub, on).clicked() {
                app.state.purpose = p;
            }
            ui.add_space(6.0);
        }
    });
    ui.add_space(4.0);
    // Shown, not a hover tip in link colour: a click on it did nothing and read as broken (#71).
    ui.scope(|ui| {
        ui.set_max_width(width);
        ui.label(RichText::new(tr("Why we ask")).size(theme::SMALL).color(t.text_dim).strong());
        ui.label(RichText::new(tr(
            "Looking inside software you own or are allowed to examine is normal and often protected (learning, compatibility, security). Saying why helps you keep to it; it stays on this computer.",
        ))
        .size(theme::SMALL)
        .color(t.text_dim));
    });
    ui.add_space(14.0);
    if widgets::primary(ui, "Continue").clicked() {
        app.state.first_run = FirstRunStep::Done;
    }
}

fn purpose_icon(p: craft_anything_engine::project::Purpose) -> &'static str {
    use craft_anything_engine::project::Purpose::*;
    match p {
        OwnOrAuthorized => "user",
        Education => "graduation-cap",
        Interoperability => "link",
        Security => "shield-check",
        Preservation => "history",
    }
}

fn done(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(64.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 32.0, t.accent);
    icons::paint(ui, rect, "check", 30.0, t.on_accent);
    ui.add_space(22.0);
    ui.label(theme::heading(tr("You're all set"), crate::make::HEADLINE));
    ui.add_space(10.0);
    let brain = app.state.backend.clone().unwrap_or_else(|| tr("the built-in engines"));
    let builds = match app.default_builder() {
        Some(b) => crate::i18n::trf("{name} builds what you make; answers come from {brain}.", &[("name", &b), ("brain", &brain)]),
        None => crate::i18n::trf("Answers come from {brain}. To build, set up a coding agent in Settings → AI.", &[("brain", &brain)]),
    };
    ui.label(RichText::new(builds).size(theme::SMALL).color(t.text_dim));
    ui.add_space(6.0);
    ui.label(
        RichText::new(tr(
            "Say what you want to make, or drop an app or file on the window to look inside it. Everything runs in Clean room mode unless you switch on Deep.",
        ))
        .size(theme::SMALL)
        .color(t.text_dim),
    );
    ui.add_space(30.0);
    if widgets::primary_icon(ui, Some("rocket"), "Let's go", true).clicked() {
        app.finish_first_run();
    }
}
