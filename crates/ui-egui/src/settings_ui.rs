//! Settings: General (mode, theme, language), Engines (Ghidra, tool paths), AI, Builds (when a
//! build is good enough, generated art), Plugins (MCP servers), Safety.

use crate::i18n::{LANGUAGES, tr, trf};
use crate::make::tint;
use crate::state::SettingsTab;
use crate::theme::{self, ThemeKind, Tokens};
use crate::{CraftApp, Pick, icons, widgets};
use craft_anything_ai::BackendKind;
use craft_anything_engine::Mode;
use egui::{Align, Layout, Margin, RichText, Stroke, Ui, Vec2};
use std::path::PathBuf;

pub fn show(app: &mut CraftApp, ctx: &egui::Context) {
    if !app.state.settings_open {
        return;
    }
    let t = Tokens::get(ctx);
    let mut open = true;
    // A sheet over a dimmed window (Esc or a click outside closes it), a fixed size so switching
    // pages doesn't make it jump: the pages on the left, the chosen one's groups on the right.
    let screen = ctx.content_rect();
    let size = Vec2::new((screen.width() - 48.0).clamp(480.0, 840.0), (screen.height() - 96.0).clamp(360.0, 660.0));
    let resp =
        egui::Modal::new(egui::Id::new("settings-window")).frame(crate::home::sheet_frame(&t, 0)).backdrop_color(crate::home::scrim(&t)).show(ctx, |ui| {
            ui.set_min_size(size);
            ui.set_max_size(size);
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(Vec2::new(NAV_W, size.y), Layout::top_down(Align::Min), |ui| {
                    ui.set_min_size(Vec2::new(NAV_W, size.y));
                    egui::Frame::new().inner_margin(Margin { left: 10, right: 10, top: 22, bottom: 10 }).show(ui, |ui| nav(app, ui));
                });
                let x = ui.cursor().left();
                ui.painter().vline(x, ui.min_rect().y_range(), Stroke::new(1.0, t.separator));
                ui.allocate_ui_with_layout(Vec2::new(size.x - NAV_W, size.y), Layout::top_down(Align::Min), |ui| {
                    ui.set_min_size(Vec2::new(size.x - NAV_W, size.y));
                    egui::Frame::new().inner_margin(Margin { left: 32, right: 18, top: 18, bottom: 0 }).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // The page's name, painted: the page list already names it for screen readers.
                            let title = TABS.iter().find(|(tab, ..)| *tab == app.state.settings_tab).map_or("", |(_, _, title)| *title);
                            let g = ui.painter().layout_no_wrap(tr(title), theme::semibold(crate::home::DIALOG_TITLE), t.text);
                            let (r, _) = ui.allocate_exact_size(Vec2::new(g.size().x, 32.0), egui::Sense::hover());
                            ui.painter().galley(egui::pos2(r.left(), r.center().y - g.size().y / 2.0), g, t.text);
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if icons::button(ui, "x", 30.0, false, "Close").clicked() {
                                    open = false;
                                }
                            });
                        });
                        // An error (a save that failed) shows here too: the message bar is behind this window.
                        if let Some(i) = app.state.toasts.iter().rposition(|(_, error)| *error) {
                            let text = app.state.toasts.get(i).map(|(m, _)| m.clone()).unwrap_or_default();
                            ui.add_space(10.0);
                            egui::Frame::new().fill(t.danger.gamma_multiply(0.12)).corner_radius(t.r()).inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;
                                    icons::show(ui, "triangle-alert", 16.0, t.danger);
                                    ui.add(egui::Label::new(RichText::new(text).color(t.danger)).wrap());
                                    if icons::button(ui, "x", 24.0, false, "Dismiss").clicked() {
                                        app.state.toasts.remove(i);
                                    }
                                });
                            });
                        }
                        ui.add_space(14.0);
                        // Exactly the page's width: a row that comes out wider must not widen the sheet (#68).
                        let page_w = ui.available_width();
                        egui::ScrollArea::vertical().auto_shrink([false, false]).max_width(page_w).show(ui, |ui| {
                            ui.set_max_width(page_w - 14.0);
                            egui::Frame::new().inner_margin(Margin { left: 0, right: 14, top: 0, bottom: 24 }).show(ui, |ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);
                                match app.state.settings_tab {
                                    SettingsTab::General => general(app, ui),
                                    SettingsTab::Engines => engines(app, ui),
                                    SettingsTab::Ai => ai(app, ui),
                                    SettingsTab::Builds => builds(app, ui),
                                    SettingsTab::Plugins => plugins(app, ui),
                                    SettingsTab::Safety => safety(app, ui),
                                }
                            });
                        });
                    });
                });
            });
        });
    if !open || resp.should_close() {
        app.state.settings_open = false;
    }
}

/// The settings pages: (page, icon, name).
const TABS: [(SettingsTab, &str, &str); 6] = [
    (SettingsTab::General, "settings", "General"),
    (SettingsTab::Engines, "cpu", "Engines"),
    (SettingsTab::Ai, "bot", "AI"),
    (SettingsTab::Builds, "hammer", "Builds"),
    (SettingsTab::Plugins, "puzzle", "Plugins"),
    (SettingsTab::Safety, "shield", "Safety"),
];

/// The page list's width.
const NAV_W: f32 = 200.0;

/// The page list: rows 32 tall, the open one tinted.
fn nav(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new(tr("Settings")).font(theme::semibold(theme::SMALL)).color(t.text_dim));
    });
    ui.add_space(14.0);
    for (tab, icon, title) in TABS {
        let on = app.state.settings_tab == tab;
        let (rect, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), egui::Sense::click());
        r.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, tr(title)));
        let h = widgets::hover_t(ui, &r);
        let fill = if on { tint(&t, 0.08) } else { tint(&t, 0.05 * h) };
        ui.painter().rect_filled(rect, t.r_sm(), fill);
        let fg = if on { t.text } else { theme::mix(t.text_dim, t.text, h) };
        icons::paint(ui, egui::Rect::from_min_size(egui::pos2(rect.left() + 10.0, rect.center().y - 8.0), Vec2::splat(16.0)), icon, 16.0, fg);
        ui.painter().text(egui::pos2(rect.left() + 36.0, rect.center().y), egui::Align2::LEFT_CENTER, tr(title), theme::medium(theme::SMALL), fg);
        if r.has_focus() {
            ui.painter().rect_stroke(rect, t.r_sm(), Stroke::new(2.0, t.text_dim), egui::StrokeKind::Inside);
        }
        if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            app.state.settings_tab = tab;
            if tab == SettingsTab::Ai && app.detected.is_none() {
                app.detect_backends();
            }
        }
        ui.add_space(2.0);
    }
}

/// A group of rows: a borderless r16 surface a step above the sheet. Rows inside are split by
/// hairlines inset 16.
fn group(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
    let t = Tokens::get(ui.ctx());
    egui::Frame::new().fill(tint(&t, if t.dark() { 0.045 } else { 0.04 })).corner_radius(t.r_lg()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 0.0;
        body(ui);
    });
}

/// A group holding free-form content (padded 16).
fn group_padded(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
    group(ui, |ui| {
        egui::Frame::new().inner_margin(Margin::same(16)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            body(ui);
        });
    });
}

/// The hairline above a row, unless it's the group's first.
fn hairline(ui: &mut Ui) {
    let top = ui.cursor().top();
    if top > ui.min_rect().top() + 0.5 {
        let t = Tokens::get(ui.ctx());
        let x = ui.max_rect().x_range();
        // One device pixel, a step brighter than the group.
        ui.painter().hline((x.min + 16.0)..=x.max, top, Stroke::new(1.0 / ui.ctx().pixels_per_point(), tint(&t, 0.09)));
    }
}

/// One row in a group: padded free-form content, split from the row above by a hairline.
fn item(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
    hairline(ui);
    egui::Frame::new().inner_margin(Margin::symmetric(16, 12)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(28.0);
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);
        body(ui);
    });
}

/// A section's title above a group.
fn section(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.add_space(22.0);
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new(tr(text)).font(theme::semibold(theme::SMALL)).color(t.text_dim));
    });
    ui.add_space(10.0);
}

/// A quiet note under a group.
fn footnote(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.add(egui::Label::new(RichText::new(tr(text)).color(t.text_faint).size(theme::CAPTION)).wrap());
    });
}

/// The space between groups.
fn gap(ui: &mut Ui) {
    ui.add_space(12.0);
}

/// A setting: its name and a line about it on the left, its control on the right.
fn row(ui: &mut Ui, title: &str, sub: &str, add: impl FnOnce(&mut Ui)) {
    let t = Tokens::get(ui.ctx());
    item(ui, |ui| {
        ui.horizontal_top(|ui| {
            let w = ui.available_width();
            let left = (w * 0.52).floor();
            ui.allocate_ui_with_layout(Vec2::new(left, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(left - 12.0);
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.label(RichText::new(tr(title)).font(theme::medium(theme::SMALL)));
                if !sub.is_empty() {
                    ui.add(egui::Label::new(RichText::new(tr(sub)).color(t.text_dim).size(theme::CAPTION)).wrap());
                }
            });
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
                add(ui);
            });
        });
    });
}

fn general(app: &mut CraftApp, ui: &mut Ui) {
    group(ui, |ui| {
        row(ui, "Mode", "Simple: say what you want and it's handled for you. Advanced: every tool, the evidence and the logs.", |ui| {
            let sel = usize::from(app.settings.mode == Mode::Experienced);
            if let Some(i) = widgets::segmented(ui, &["Simple", "Advanced"], sel, 30.0) {
                app.set_mode(if i == 0 { Mode::Beginner } else { Mode::Experienced });
            }
        });
        row(ui, "Theme", "", |ui| {
            let sel = usize::from(app.theme_kind() == ThemeKind::StudioLight);
            if let Some(i) = widgets::segmented(ui, &["Studio", "Studio Light"], sel, 30.0) {
                app.settings.theme = ThemeKind::ALL.get(i).copied().unwrap_or_default().id().into();
                app.settings_changed();
            }
        });
        // A choice of one isn't a choice: the row appears once there's a second language.
        if LANGUAGES.len() > 1 {
            language(app, ui);
        }
    });
    section(ui, "Projects");
    group(ui, |ui| {
        let dir = app.settings.projects_dir();
        row(ui, "New projects go in", &dir.display().to_string(), |ui| {
            if widgets::secondary(ui, Some("folder-open"), "Change…").clicked() {
                app.pick(crate::Pick::ProjectsDir);
            }
        });
        row(
            ui,
            "Also list projects from",
            "Folders of projects, and projects started in a folder you picked. Removing one only takes it off the list.",
            |ui| {
                if widgets::secondary(ui, Some("plus"), "Add a folder…").clicked() {
                    app.pick(crate::Pick::ExtraProjectsDir);
                }
            },
        );
        let mut remove = None;
        for f in &app.settings.project_folders {
            item(ui, |ui| {
                ui.horizontal(|ui| {
                    let t = Tokens::get(ui.ctx());
                    crate::icons::show(ui, "folder", 14.0, t.text_faint);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if crate::icons::button(ui, "x", 24.0, false, &trf("Stop listing {folder}", &[("folder", &f.display().to_string())])).clicked() {
                            remove = Some(f.clone());
                        }
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.add(egui::Label::new(RichText::new(f.display().to_string()).font(theme::mono(12.0))).truncate());
                        });
                    });
                });
            });
        }
        if let Some(f) = remove {
            app.remove_project_folder(&f);
        }
        row(ui, "When a folder you pick has files", "Whether they're the app to edit in place, or material the app starts from beside them.", |ui| {
            let current = crate::folders::EXISTING_CHOICES.iter().find(|(_, v)| *v == app.settings.existing_files).map_or("Ask each time", |(l, _)| *l);
            crate::make::pill_combos(ui);
            egui::ComboBox::from_id_salt("existing-files").icon(widgets::combo_icon).selected_text(RichText::new(tr(current)).size(theme::SMALL)).show_ui(
                ui,
                |ui| {
                    for (label, value) in crate::folders::EXISTING_CHOICES {
                        if ui.selectable_label(app.settings.existing_files == value, tr(label)).clicked() {
                            app.settings.existing_files = value;
                            app.settings_changed();
                        }
                    }
                },
            );
        });
    });
    gap(ui);
    group(ui, |ui| {
        row(ui, "Show the welcome again", "Runs the first-run steps next time.", |ui| {
            if widgets::secondary(ui, Some("refresh-cw"), "Reset").clicked() {
                app.settings.first_run_done = false;
                app.state.first_run = crate::state::FirstRunStep::Welcome;
                app.state.settings_open = false;
                app.settings_changed();
            }
        });
    });
}

fn language(app: &mut CraftApp, ui: &mut Ui) {
    row(ui, "Language", "", |ui| {
        let selected = (!app.settings.language.trim().is_empty()).then(|| crate::i18n::language_from_tag(&app.settings.language).unwrap_or("en"));
        let current = tr(LANGUAGES.iter().find(|l| Some(l.code) == selected).map(|l| l.name).unwrap_or("System"));
        crate::make::pill_combos(ui);
        egui::ComboBox::from_id_salt("lang").icon(widgets::combo_icon).selected_text(RichText::new(current).size(theme::SMALL)).show_ui(ui, |ui| {
            if ui.selectable_label(app.settings.language.trim().is_empty(), tr("System")).clicked() {
                app.settings.language.clear();
                crate::i18n::set_language("");
                app.settings_changed();
            }
            for l in LANGUAGES {
                if ui.selectable_label(selected == Some(l.code), l.name).clicked() {
                    app.settings.language = l.code.into();
                    crate::i18n::set_language(l.code);
                    app.settings_changed();
                }
            }
        });
    });
}

/// A path setting; `name` is what a screen reader calls it.
fn path_field(ui: &mut Ui, value: &mut Option<PathBuf>, name: &str, hint: &str) -> bool {
    // While typing, the text as typed: the path kept is trimmed, and showing that instead would
    // drop the space typed in "C:\Program Files" before the next letter.
    let typed = ui.next_auto_id().with("typed");
    let shown = value.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
    let mut s = ui.data(|d| d.get_temp::<String>(typed)).unwrap_or(shown);
    let t = Tokens::get(ui.ctx());
    let r = ui.add(
        egui::TextEdit::singleline(&mut s)
            .hint_text(RichText::new(tr(hint)).font(theme::mono(12.0)))
            // Up to 240, leaving room for a Choose… button after it, so a row never widens the window (#68).
            .desired_width((ui.available_width() - 34.0).clamp(120.0, 240.0))
            .font(theme::mono(12.0))
            .frame(widgets::text_field_frame_compact(&t).corner_radius(t.r())),
    );
    widgets::named(r.clone(), &tr(name));
    if r.has_focus() {
        ui.data_mut(|d| d.insert_temp(typed, s.clone()));
    } else {
        ui.data_mut(|d| d.remove::<String>(typed));
    }
    if r.changed() {
        *value = if s.trim().is_empty() { None } else { Some(PathBuf::from(s.trim())) };
        return true;
    }
    false
}

fn engines(app: &mut CraftApp, ui: &mut Ui) {
    group(ui, |ui| {
        row(
            ui,
            "Clean room / Deep",
            "Clean room never reads the original's code, files or page source: it works from what you can see. Deep reads them with every tool, and with Ghidra when it's installed. Every finding records which was used.",
            |ui| {
                let sel = usize::from(app.settings.deep_mode);
                if let Some(i) = widgets::segmented(ui, &["Clean room", "Deep"], sel, 30.0) {
                    app.set_deep(i == 1);
                }
            },
        );
    });
    section(ui, "Ghidra");
    group(ui, |ui| {
        row(ui, "Ghidra", "Adds its decompiler to Deep mode. Craft Anything can download and set it up for you.", |ui| {
            crate::engine_ui::ghidra_status(app, ui);
        });
        row(
            ui,
            "Memory for Ghidra",
            "More memory makes Ghidra's first look at a big program faster; leave some for the rest of the computer. Used from the next time Ghidra starts.",
            |ui| crate::engine_ui::ghidra_memory_setting(app, ui),
        );
        row(ui, "Ghidra folder", "Leave empty to use the one Craft Anything sets up.", |ui| {
            if path_field(ui, &mut app.settings.engines.ghidra.install_dir, "Ghidra folder", "managed") {
                app.settings_changed();
            }
            if icons::button(ui, "folder-open", 28.0, false, "Choose…").clicked() {
                app.pick(Pick::GhidraDir);
            }
        });
        row(ui, "Java runtime", "Only for Ghidra. Empty = managed.", |ui| {
            if path_field(ui, &mut app.settings.engines.ghidra.java_home, "Java runtime", "managed") {
                app.settings_changed();
            }
        });
    });
    footnote(ui, &crate::engine_ui::ghidra_memory_guide());
    gap(ui);
    group_padded(ui, |ui| crate::engine_ui::settings_section(app, ui));
    section(ui, "Your own tool installs (optional)");
    let mut changed = false;
    group(ui, |ui| {
        // Names and purposes come from the engine table; the override list matches it (tested).
        for (id, v) in app.settings.engines.tools.entries_mut() {
            let (name, needed_for) = craft_anything_engines::engine(id).map(|e| (e.name, e.needed_for)).unwrap_or((id, ""));
            item(ui, |ui| {
                ui.horizontal(|ui| {
                    let w = ui.available_width();
                    let left = (w * 0.52).floor();
                    ui.allocate_ui_with_layout(Vec2::new(left, 0.0), Layout::left_to_right(Align::Center), |ui| {
                        ui.set_width(left - 12.0);
                        ui.label(RichText::new(tr(name)).size(theme::SMALL)).on_hover_text(format!("{}\n{id}", tr(needed_for)));
                    });
                    changed |= path_field(ui, v, name, "managed");
                });
            });
        }
    });
    footnote(ui, "Leave a path empty to use the managed install. We never need environment variables.");
    if changed {
        app.settings_changed();
    }
}

fn ai(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    group_padded(ui, |ui| crate::engine_ui::assets_settings(app, ui));
    section(ui, "Your brains");
    if app.backends.is_empty() {
        group(ui, |ui| {
            item(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(tr("No AI is set up yet. The built-in engines still work; an AI adds plain-words answers.")).color(t.text_dim),
                    )
                    .wrap(),
                );
            });
        });
    }
    let (mut pick, mut builds, mut extracts, mut toggle, mut remove) = (None, None, None, None, None);
    let (mut choose_program, mut usual_program) = (None, None);
    // The AI whose Remove was pressed once: asked again before its key is deleted.
    let confirm_id = egui::Id::new("remove-ai");
    let confirming: Option<String> = ui.data(|d| d.get_temp(confirm_id));
    if !app.backends.is_empty() {
        group(ui, |ui| {
            for b in &app.backends {
                item(ui, |ui| {
                    // The name and model on their own row (shortened, full text on hover), the buttons on a
                    // row below that wraps: nothing overlaps however long the name or narrow the window.
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 15.0, tint(&t, 0.08));
                        icons::paint(ui, rect, "bot", 16.0, if b.enabled { t.text } else { t.text_faint });
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.add(egui::Label::new(RichText::new(&b.name).font(theme::medium(theme::SMALL))).truncate());
                            ui.add(
                                egui::Label::new(
                                    RichText::new(format!(
                                        "{:?} · {}{}",
                                        b.kind,
                                        b.model,
                                        b.ssh_host.as_ref().map(|h| format!(" · ssh {h}")).unwrap_or_default()
                                    ))
                                    .color(t.text_faint)
                                    .size(theme::CAPTION),
                                )
                                .truncate(),
                            );
                            // An agent app on this computer: which program runs it, for one installed
                            // where it isn't looked for (#53).
                            if b.kind == BackendKind::AgentCli && b.ssh_host.as_deref().is_none_or(|h| h.trim().is_empty()) {
                                let program = app.agent_programs.get(&b.name);
                                ui.horizontal_wrapped(|ui| {
                                    // The links at the caption's size, so the line reads as one.
                                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(theme::CAPTION));
                                    let text = program.map_or_else(|| tr("Program: found automatically"), |p| trf("Program: {path}", &[("path", p)]));
                                    ui.add(egui::Label::new(RichText::new(text).color(t.text_faint).size(theme::CAPTION)).truncate());
                                    if widgets::link(ui, "Choose…").clicked() {
                                        choose_program = Some(b.name.clone());
                                    }
                                    if program.is_some() && widgets::link(ui, "Find it automatically").clicked() {
                                        usual_program = Some(b.name.clone());
                                    }
                                });
                            }
                        });
                    });
                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                        ui.add_space(36.0);
                        if app.state.backend.as_ref() == Some(&b.name) {
                            widgets::pill(ui, "In use", t.on_accent, t.accent);
                        } else if widgets::secondary(ui, None, "Use").clicked() {
                            pick = Some(b.name.clone());
                        }
                        // Who builds: the chosen one says so (click to go back to automatic).
                        if app.builder.as_ref() == Some(&b.name) {
                            if widgets::secondary(ui, Some("hammer"), "Builds")
                                .on_hover_text(tr("Builds use this one. Click to choose automatically again."))
                                .clicked()
                            {
                                builds = Some(None);
                            }
                        } else if b.enabled && widgets::secondary(ui, None, "Build with this").clicked() {
                            builds = Some(Some(b.name.clone()));
                        }
                        // Who takes a game's files out in Deep mode (an API backend, e.g. Kimi K3).
                        if b.kind != BackendKind::AgentCli {
                            if app.extractor.as_ref() == Some(&b.name) {
                                if widgets::secondary(ui, Some("package"), "Extracts")
                                    .on_hover_text(tr("In Deep mode this one takes a game's own files out after the study. Click to stop."))
                                    .clicked()
                                {
                                    extracts = Some(None);
                                }
                            } else if b.enabled && widgets::secondary(ui, None, "Extract with this").clicked() {
                                extracts = Some(Some(b.name.clone()));
                            }
                        }
                        // Off: kept with its key, but nothing uses it.
                        let (label, tip) = if b.enabled {
                            ("Turn off", "Nothing uses it until it's on again; it keeps its key.")
                        } else {
                            ("Turn on", "Builds and answers may use it again.")
                        };
                        if widgets::secondary(ui, None, label).on_hover_text(tr(tip)).clicked() {
                            toggle = Some((b.name.clone(), !b.enabled));
                        }
                        if confirming.as_ref() == Some(&b.name) {
                            if widgets::secondary(ui, Some("trash-2"), "Remove it and its key").clicked() {
                                remove = Some(b.name.clone());
                            }
                            if crate::make::quiet_link(ui, "Keep it").clicked() {
                                ui.data_mut(|d| d.remove::<String>(confirm_id));
                            }
                        } else if icons::button(ui, "trash-2", 28.0, false, &trf("Remove {name}", &[("name", &b.name)])).clicked() {
                            ui.data_mut(|d| d.insert_temp(confirm_id, b.name.clone()));
                        }
                    });
                    if !b.enabled {
                        ui.horizontal(|ui| {
                            ui.add_space(36.0);
                            ui.label(RichText::new(tr("Off")).color(t.text_faint).size(theme::CAPTION));
                        });
                    }
                });
            }
        });
    }
    if let Some((name, on)) = toggle {
        app.set_backend_enabled(&name, on);
    }
    if let Some(name) = choose_program {
        app.picking_program_for = Some(name);
        app.pick(Pick::AgentProgram);
    }
    if let Some(name) = usual_program {
        app.set_agent_program(&name, None);
    }
    if let Some(name) = remove {
        ui.data_mut(|d| d.remove::<String>(confirm_id));
        app.remove_backend(&name);
    }
    if let Some(p) = pick {
        app.choose_backend(Some(p));
    }
    if let Some(b) = builds {
        app.set_role(craft_anything_ai::Role::Builder, b);
    }
    if let Some(e) = extracts {
        app.set_role(craft_anything_ai::Role::Extractor, e);
    }
    ui.add_space(22.0);
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new(tr("Found on this computer")).font(theme::semibold(theme::SMALL)).color(t.text_dim));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.detecting {
                ui.add(egui::Spinner::new().color(t.text_dim));
            } else if widgets::secondary(ui, Some("radar"), "Detect").clicked() {
                app.detect_backends();
            }
        });
    });
    ui.add_space(10.0);
    let detected = app.detected.clone().unwrap_or_default();
    if detected.is_empty() && !app.detecting {
        group(ui, |ui| {
            item(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(tr("Nothing found yet. We look for API keys, local model servers (Ollama, LM Studio) and agent apps you already have."))
                            .color(t.text_dim),
                    )
                    .wrap(),
                );
            });
        });
    }
    if detected.is_empty() {
        return;
    }
    group(ui, |ui| {
        for d in detected {
            let configured = app.backends.iter().any(|b| b.name == d.suggestion.name);
            item(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new(&d.suggestion.name).font(theme::medium(theme::SMALL)));
                        ui.label(RichText::new(&d.reason).color(t.text_faint).size(theme::CAPTION));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if configured {
                            widgets::pill(ui, "Added", t.success, tint(&t, 0.08));
                        } else if !d.needs_key && widgets::primary(ui, "Add").clicked() {
                            app.add_backend(d.suggestion.clone(), None);
                        }
                    });
                });
                if d.needs_key && !configured {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let key = app.key_inputs.entry(d.suggestion.name.clone()).or_default();
                        ui.add(
                            egui::TextEdit::singleline(key)
                                .password(true)
                                .hint_text(tr("API key (stored in your system keychain)"))
                                .desired_width(260.0)
                                .frame(widgets::text_field_frame_compact(&t).corner_radius(t.r())),
                        );
                        let k = key.trim().to_string();
                        if widgets::primary_icon(ui, None, "Add", !k.is_empty()).clicked() && !k.is_empty() {
                            // The key goes to the keychain; the app keeps no copy of it.
                            app.key_inputs.remove(&d.suggestion.name);
                            app.add_backend(d.suggestion.clone(), Some(k));
                        }
                    });
                }
            });
        }
    });
}

/// A number field; true when it changed.
fn number<T: egui::emath::Numeric>(ui: &mut Ui, value: &mut T, range: std::ops::RangeInclusive<T>, prefix: &str, suffix: &str) -> bool {
    ui.add(egui::DragValue::new(value).range(range).prefix(prefix).suffix(suffix)).changed()
}

fn builds(app: &mut CraftApp, ui: &mut Ui) {
    let mut changed = false;
    let b = &mut app.settings.build;
    group(ui, |ui| {
        row(ui, "Good enough", "How alike a rebuild must look to the original before refining stops.", |ui| {
            changed |= number(ui, &mut b.good_enough, 50.0..=100.0, "", " %");
        });
        row(ui, "Refining rounds", "At most this many rounds of fixes after the first build.", |ui| {
            changed |= number(ui, &mut b.max_rounds, 0..=20, "", "");
        });
        row(ui, "Website pages", "Other pages of a website studied and compared besides the one you gave.", |ui| {
            changed |= number(ui, &mut b.max_pages, 0..=craft_anything_ai::rebuild::kinds::website::MAX_ROUTES, "", "");
        });
        row(ui, "Match my pictures", "Apps and games: keep refining until they look this alike to your pictures. Off, the likeness is only reported.", |ui| {
            let mut on = b.picture_good_enough.is_some();
            if widgets::toggle(ui, &mut on, "Use as a goal").changed() {
                b.picture_good_enough = on.then_some(80.0);
                changed = true;
            }
            if let Some(v) = &mut b.picture_good_enough {
                changed |= number(ui, v, 30.0..=100.0, "", " %");
            }
        });
        row(ui, "Builds at once", "How many projects may build at the same time. More wait their turn.", |ui| {
            changed |= number(ui, &mut b.parallel, 1..=8, "", "");
        });
    });
    section(ui, "Generated art (games)");
    let a = &mut app.settings.assets;
    group(ui, |ui| {
        row(
            ui,
            "Budget for a new project",
            "What a new game may spend on generated pictures and 3D models before you're asked. Each project keeps its own.",
            |ui| {
                changed |= number(ui, &mut a.budget_usd, 0.0..=10_000.0, "$", "");
            },
        );
        let table = craft_anything_engines::assets::Catalogue::built_in();
        row(ui, "Picture model", "What pictures and textures are made with.", |ui| {
            let current = table.image_model(a.image_model.as_deref()).or_else(|_| table.image_model(None)).map(|m| m.name.clone()).unwrap_or_default();
            crate::make::pill_combos(ui);
            egui::ComboBox::from_id_salt("image-model").icon(widgets::combo_icon).selected_text(RichText::new(current).size(theme::SMALL)).show_ui(ui, |ui| {
                for m in table.images() {
                    let price = crate::usage::asset_cost_words(m.cost(craft_anything_engines::assets::Shape::Square));
                    let label = format!("{} · {} {}", m.name, price, tr("a picture"));
                    let on = a.image_model.as_deref() == Some(m.id.as_str());
                    if ui.selectable_label(on, label).on_hover_text(&m.licence).clicked() && !on {
                        a.image_model = Some(m.id.clone());
                        changed = true;
                    }
                }
            });
        });
    });
    if changed {
        app.settings_changed();
    }
}

/// Plugins: external MCP servers (`[[plugins.mcp]]`) and WASM components (`[[plugins.wasm]]`),
/// listed, turned on or off, added and removed.
fn plugins(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.add(
        egui::Label::new(
            RichText::new(tr(
                "Plugins add tools Craft Anything offers as its own: MCP servers (a command or an address; read their tools after adding) or WebAssembly components (a .wasm file, which can't touch your files or the network).",
            ))
            .color(t.text_dim)
            .size(theme::CAPTION),
        )
        .wrap(),
    );
    ui.add_space(14.0);
    let plugins = app.settings.plugins.mcp.clone();
    let wasm = app.settings.plugins.wasm.clone();
    let mut changed = false;
    group(ui, |ui| {
        if plugins.is_empty() && wasm.is_empty() {
            item(ui, |ui| {
                ui.label(RichText::new(tr("No plugins yet.")).color(t.text_dim));
            });
        }
        for (i, p) in plugins.iter().enumerate() {
            item(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width((ui.available_width() - 150.0).max(160.0));
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new(p.display_name()).font(theme::medium(theme::SMALL)));
                        ui.label(RichText::new(p.place()).font(theme::mono(11.5)).color(t.text_faint));
                        if let Some(problem) = p.problem() {
                            ui.label(RichText::new(problem).color(t.warning).size(theme::CAPTION));
                        }
                        if !p.consent {
                            ui.label(RichText::new(tr("Runs its tools without asking")).color(t.warning).size(theme::CAPTION));
                        }
                    });
                    if let Some(entry) = app.settings.plugins.mcp.get_mut(i)
                        && widgets::toggle(ui, &mut entry.enabled, "On").changed()
                    {
                        changed = true;
                    }
                    if icons::button(ui, "refresh-cw", 28.0, false, "Read its tools").clicked() {
                        app.act("plugins_refresh", serde_json::json!({"id": p.id}), tr("Read the plugin's tools"));
                    }
                    if icons::button(ui, "trash-2", 28.0, false, "Remove").clicked() {
                        app.act("plugins_remove", serde_json::json!({"id": p.id}), tr("Remove a plugin"));
                    }
                });
            });
        }
        for (i, p) in wasm.iter().enumerate() {
            item(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width((ui.available_width() - 150.0).max(160.0));
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new(p.display_name()).font(theme::medium(theme::SMALL)));
                        ui.label(RichText::new(p.path.display().to_string()).font(theme::mono(11.5)).color(t.text_faint));
                        if let Some(problem) = p.problem() {
                            ui.label(RichText::new(problem).color(t.warning).size(theme::CAPTION));
                        }
                    });
                    if let Some(entry) = app.settings.plugins.wasm.get_mut(i)
                        && widgets::toggle(ui, &mut entry.enabled, "On").changed()
                    {
                        changed = true;
                    }
                    if icons::button(ui, "trash-2", 28.0, false, "Remove").clicked() {
                        app.act("plugins_remove", serde_json::json!({"id": p.id}), tr("Remove a plugin"));
                    }
                });
            });
        }
    });
    if changed {
        app.settings_changed();
    }
    if !plugins.is_empty() || !wasm.is_empty() {
        footnote(ui, "Turning a plugin on or off takes effect from the next start.");
    }
    section(ui, "Add a plugin");
    group_padded(ui, |ui| {
        let form = &mut app.state.plugin_form;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            let id = ui.add(
                egui::TextEdit::singleline(&mut form.id)
                    .hint_text(tr("id, e.g. docs"))
                    .desired_width(110.0)
                    .frame(widgets::text_field_frame_compact(&t).corner_radius(t.r())),
            );
            widgets::named(id, "Plugin id");
            let place = ui.add(
                egui::TextEdit::singleline(&mut form.place)
                    .hint_text(tr("command, https:// address or .wasm file"))
                    .desired_width(220.0)
                    .frame(widgets::text_field_frame_compact(&t).corner_radius(t.r())),
            );
            widgets::named(place, "Plugin command or address");
            let args = ui.add(
                egui::TextEdit::singleline(&mut form.args)
                    .hint_text(tr("arguments (optional)"))
                    .desired_width(160.0)
                    .frame(widgets::text_field_frame_compact(&t).corner_radius(t.r())),
            );
            widgets::named(args, "Plugin arguments");
        });
        ui.add_space(8.0);
        let params = app.state.plugin_form.params();
        if widgets::primary_icon(ui, Some("plus"), "Add", params.is_some()).clicked()
            && let Some(params) = params
        {
            app.act("plugins_add", params, tr("Add a plugin"));
            app.state.plugin_form = Default::default();
        }
    });
    footnote(ui, "Plugins added here always ask before each call.");
}

fn safety(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    group(ui, |ui| {
        row(
            ui,
            "Ask before running anything",
            "Tools that run code from a target or change a phone always ask first, unless you allowed them for a project.",
            |ui| {
                if widgets::toggle(ui, &mut app.settings.ask_before_running, "Ask first").changed() {
                    app.settings_changed();
                }
            },
        );
        if !app.settings.ask_before_running {
            item(ui, |ui| {
                ui.horizontal(|ui| {
                    icons::show(ui, "triangle-alert", 15.0, t.warning);
                    ui.add(
                        egui::Label::new(
                            RichText::new(tr("Tools may now run code from what you open without asking. Turn this back on unless you're sure."))
                                .color(t.warning),
                        )
                        .wrap(),
                    );
                });
            });
        }
        row(
            ui,
            "Auto mode",
            "Builds keep going without stopping to ask you before an action. The builder still never deletes outside its project, changes system settings or publishes anything, and stays within each project's asset budget.",
            |ui| {
                if widgets::toggle(ui, &mut app.settings.auto_mode, "Keep going without asking").changed() {
                    app.settings_changed();
                }
            },
        );
        row(
            ui,
            "The builder's questions",
            "When the builder needs a choice made: show it to you and wait, or let it decide and say what it chose. Each project can choose for itself under its prompt box.",
            |ui| {
                use craft_anything_engine::workspace::Questions;
                let sel = usize::from(app.settings.questions() == Questions::Decide);
                if let Some(i) = widgets::segmented(ui, &["Ask me", "Decide yourself"], sel, 30.0) {
                    app.settings.questions = Some(if i == 0 { Questions::Ask } else { Questions::Decide });
                    app.settings_changed();
                }
            },
        );
        row(
            ui,
            "Study apps by running them",
            "When you rebuild an app or game, start it once to picture its window and compare the rebuild with it. It runs that program.",
            |ui| {
                if widgets::toggle(ui, &mut app.settings.runtime_study, "Run to study").changed() {
                    app.settings_changed();
                }
            },
        );
    });
    if let Some(p) = app.view.project.clone() {
        section(ui, &format!("{} {}", tr("Allowed in"), p));
        group(ui, |ui| {
            if app.view.project_allows.is_empty() {
                item(ui, |ui| {
                    ui.label(RichText::new(tr("Nothing allowed yet.")).color(t.text_dim));
                });
            }
            for tool in app.view.project_allows.clone() {
                item(ui, |ui| {
                    ui.label(RichText::new(tool).font(theme::mono(12.0)));
                });
            }
        });
    }
}
