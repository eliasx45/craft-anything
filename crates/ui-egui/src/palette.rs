//! Command palette (Ctrl/⌘ K): app commands plus every registry tool. Picking a tool opens a
//! params form generated from its schema.

use crate::i18n::tr;
use crate::state::Screen;
use crate::theme::{self, Tokens};
use crate::{CraftApp, Pick, Route, icons, schema_form, widgets};
use craft_anything_engine::{Consent, Mode};
use egui::{Margin, RichText, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppCommand {
    OpenFile,
    OpenFolder,
    ToggleMode,
    ToggleDeep,
    ToggleTheme,
    Settings,
    ExportReport,
    Home,
}

const APP_COMMANDS: &[(AppCommand, &str, &str)] = &[
    (AppCommand::OpenFile, "Open a file…", "folder-open"),
    (AppCommand::OpenFolder, "Open a folder or app bundle…", "folder"),
    (AppCommand::ToggleMode, "Switch Beginner / Experienced", "graduation-cap"),
    (AppCommand::ToggleDeep, "Switch Clean room / Deep", "microscope"),
    (AppCommand::ToggleTheme, "Switch light / dark theme", "moon"),
    (AppCommand::Settings, "Settings", "settings"),
    (AppCommand::ExportReport, "Export a report (Markdown)", "download"),
    (AppCommand::Home, "Go home", "house"),
];

enum Entry {
    App(AppCommand, &'static str, &'static str),
    Tool { id: &'static str, title: &'static str, summary: &'static str, toolset: &'static str },
}

/// Fuzzy score: every query char must appear in order; earlier and contiguous matches score higher.
pub fn score(hay: &str, query: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let hay = hay.to_lowercase();
    let mut score = 0i32;
    let mut last: Option<usize> = None;
    let mut chars = hay.char_indices();
    for q in query.to_lowercase().chars().filter(|c| !c.is_whitespace()) {
        let (i, _) = chars.by_ref().find(|(_, c)| *c == q)?;
        score += match last {
            Some(l) if i == l + 1 => 8,
            _ => 1,
        };
        if i == 0 {
            score += 6;
        }
        last = Some(i);
    }
    if hay.contains(&query.to_lowercase()) {
        score += 30;
    }
    Some(score - (hay.len() as i32 / 16))
}

pub fn show(app: &mut CraftApp, ctx: &egui::Context) {
    if !app.state.palette.open {
        return;
    }
    let t = Tokens::get(ctx);
    if let Some(tool) = app.state.palette.form_tool.clone() {
        form(app, ctx, &tool);
        return;
    }
    let entries: Vec<Entry> = APP_COMMANDS
        .iter()
        .map(|(c, title, icon)| Entry::App(*c, title, icon))
        .chain(app.engine.registry().all().iter().map(|s| Entry::Tool { id: s.id, title: s.title, summary: s.summary, toolset: s.toolset.title() }))
        .collect();
    let q = app.state.palette.query.clone();
    let mut ranked: Vec<(i32, &Entry)> = entries
        .iter()
        .filter_map(|e| {
            let hay = match e {
                Entry::App(_, title, _) => tr(title),
                Entry::Tool { id, title, .. } => format!("{} {title} {id}", tr(title)),
            };
            score(&hay, &q).map(|s| (s, e))
        })
        .collect();
    ranked.sort_by_key(|(s, _)| -s);
    ranked.truncate(60);
    let n = ranked.len();
    let (down, up, enter) = ctx.input_mut(|i| {
        (i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown), i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp), i.key_pressed(egui::Key::Enter))
    });
    let p = &mut app.state.palette;
    if down {
        p.selected = (p.selected + 1).min(n.saturating_sub(1));
    }
    if up {
        p.selected = p.selected.saturating_sub(1);
    }
    p.selected = p.selected.min(n.saturating_sub(1));
    let mut chosen: Option<usize> = None;
    let screen = ctx.content_rect();
    let width = 620.0_f32.min(screen.width() - 40.0);
    let pos = egui::pos2(screen.center().x - width / 2.0, screen.top() + 80.0);
    // Scrim
    ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("palette-scrim"))).rect_filled(screen, 0.0, crate::home::scrim(&t));
    // A popover: r16 on the raised fill, a faint inner outline and a soft deep shadow.
    let resp = egui::Area::new(egui::Id::new("palette")).order(egui::Order::Foreground).fixed_pos(pos).show(ctx, |ui| {
        egui::Frame::new()
            .fill(t.raised)
            .stroke(egui::Stroke::new(1.0, t.chrome_stroke))
            .corner_radius(t.r_lg())
            .shadow(egui::Shadow { offset: [0, 8], blur: 40, spread: 0, color: egui::Color32::from_black_alpha(if t.dark() { 64 } else { 36 }) })
            .inner_margin(Margin::same(6))
            .show(ui, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.spacing_mut().item_spacing.x = 10.0;
                    icons::show(ui, "search", 16.0, t.text_dim);
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut app.state.palette.query)
                            .hint_text(tr("Type a command or tool…"))
                            .frame(egui::Frame::NONE)
                            .desired_width(f32::INFINITY)
                            .font(egui::FontId::proportional(theme::BODY)),
                    );
                    r.request_focus();
                    if r.changed() {
                        app.state.palette.selected = 0;
                    }
                });
                ui.add_space(8.0);
                // A divider: 0.5 px, inset 10.
                let y = ui.cursor().top();
                let x = ui.max_rect().x_range();
                ui.painter().hline((x.min + 10.0)..=(x.max - 10.0), y, egui::Stroke::new(0.5, crate::make::tint(&t, 0.10)));
                ui.add_space(6.0);
                egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    if ranked.is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.label(RichText::new(tr("Nothing matches.")).color(t.text_faint));
                        });
                    }
                    for (i, (_, e)) in ranked.iter().enumerate() {
                        let on = i == app.state.palette.selected;
                        let (rect, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), egui::Sense::click());
                        // Menu rows: no fill at rest, a soft tint on the keyboard's row and under the pointer.
                        if on || r.hovered() {
                            ui.painter().rect_filled(rect, t.r_sm(), crate::make::tint(&t, if on { 0.07 } else { 0.05 }));
                        }
                        if on {
                            r.scroll_to_me(None);
                        }
                        let (icon, title, sub, tag) = match e {
                            Entry::App(_, title, icon) => (*icon, tr(title), String::new(), tr("App")),
                            Entry::Tool { id, title, summary, toolset } => ("wrench", tr(title), format!("{id} · {summary}"), tr(toolset)),
                        };
                        r.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, &title));
                        icons::paint(
                            ui,
                            egui::Rect::from_center_size(egui::pos2(rect.left() + 19.0, rect.center().y), Vec2::splat(16.0)),
                            icon,
                            15.0,
                            if on { t.text } else { t.text_dim },
                        );
                        // The tag at the right, then the title and (faint, cut to fit) what it does.
                        let tag_g = ui.painter().layout_no_wrap(tag, egui::FontId::proportional(theme::TINY), t.text_dim);
                        let tag_r = egui::Rect::from_center_size(
                            egui::pos2(rect.right() - 12.0 - tag_g.size().x / 2.0 - 7.0, rect.center().y),
                            tag_g.size() + Vec2::new(14.0, 4.0),
                        );
                        ui.painter().rect_filled(tag_r, t.r_pill(), crate::make::tint(&t, 0.075));
                        ui.painter().galley(tag_r.center() - tag_g.size() / 2.0, tag_g, t.text_dim);
                        let text_x = rect.left() + 38.0;
                        let clip = egui::Rect::from_min_max(egui::pos2(text_x, rect.top()), egui::pos2(tag_r.left() - 12.0, rect.bottom()));
                        let title_g = ui.painter().layout_no_wrap(title, egui::FontId::proportional(theme::CAPTION + 1.0), t.text);
                        let title_w = title_g.size().x;
                        ui.painter().with_clip_rect(clip).galley(egui::pos2(text_x, rect.center().y - title_g.size().y / 2.0), title_g, t.text);
                        if !sub.is_empty() {
                            let sub_g =
                                ui.painter().layout_no_wrap(crate::evidence_ui::truncate(&sub, 90), egui::FontId::proportional(theme::TINY), t.text_faint);
                            ui.painter().with_clip_rect(clip).galley(
                                egui::pos2(text_x + title_w + 10.0, rect.center().y - sub_g.size().y / 2.0),
                                sub_g,
                                t.text_faint,
                            );
                        }
                        if r.clicked() {
                            chosen = Some(i);
                        }
                    }
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new(tr("↑↓ to move · Enter to run · Esc to close")).color(t.text_faint).size(theme::TINY));
                });
                ui.add_space(4.0);
            });
    });
    if enter && n > 0 {
        chosen = Some(app.state.palette.selected);
    }
    if resp.response.clicked_elsewhere() && chosen.is_none() {
        app.state.palette.open = false;
    }
    let Some(i) = chosen else { return };
    match ranked.get(i).map(|(_, e)| *e) {
        Some(Entry::App(c, _, _)) => {
            app.state.palette.open = false;
            run_app_command(app, *c);
        }
        Some(Entry::Tool { id, .. }) => {
            let id = id.to_string();
            let schema = app.engine.registry().get(&id).map(|s| app.engine.registry().input_schema(s)).unwrap_or_default();
            if schema_form::fields(&schema).is_empty() {
                app.state.palette.open = false;
                app.run_tool(&id, serde_json::json!({}), Route::Palette, Consent::NotAsked);
                app.set_mode(Mode::Experienced);
            } else {
                app.state.palette.form_tool = Some(id);
                app.state.palette.form.clear();
                app.state.palette.form_error = None;
            }
        }
        None => {}
    }
}

fn run_app_command(app: &mut CraftApp, c: AppCommand) {
    match c {
        AppCommand::OpenFile => app.pick(Pick::OpenFile),
        AppCommand::OpenFolder => app.pick(Pick::OpenFolder),
        AppCommand::ToggleMode => {
            let m = if app.beginner() { Mode::Experienced } else { Mode::Beginner };
            app.set_mode(m);
        }
        AppCommand::ToggleDeep => {
            let on = !app.settings.deep_mode;
            app.set_deep(on);
        }
        AppCommand::ToggleTheme => {
            let next = if app.theme_kind() == theme::ThemeKind::Studio { theme::ThemeKind::StudioLight } else { theme::ThemeKind::Studio };
            app.settings.theme = next.id().into();
            app.settings_changed();
        }
        AppCommand::Settings => app.state.settings_open = true,
        AppCommand::ExportReport => app.export_report(),
        AppCommand::Home => {
            app.set_mode(Mode::Beginner);
            app.state.screen = Screen::Home;
        }
    }
}

fn form(app: &mut CraftApp, ctx: &egui::Context, tool: &str) {
    let t = Tokens::get(ctx);
    let Some(spec) = app.engine.registry().get(tool).copied() else {
        app.state.palette.form_tool = None;
        return;
    };
    let fields = schema_form::fields(&app.engine.registry().input_schema(&spec));
    let mut run = false;
    let mut back = false;
    let mut open = true;
    egui::Window::new(RichText::new(tr(spec.title)).font(theme::semibold(theme::BODY)))
        .id(egui::Id::new("tool-form"))
        .frame(crate::home::sheet_frame(&t, 20))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(520.0)
        .anchor(egui::Align2::CENTER_TOP, Vec2::new(0.0, 80.0))
        .show(ctx, |ui| {
            ui.label(RichText::new(spec.summary).color(t.text_dim));
            ui.label(RichText::new(spec.id).font(theme::mono(11.5)).color(t.text_faint));
            if spec.effects.needs_consent() {
                ui.horizontal(|ui| {
                    icons::show(ui, "shield", 14.0, t.warning);
                    ui.label(RichText::new(tr("This runs code from the target or changes a device. Clicking Run is your OK.")).color(t.warning));
                });
            }
            ui.add_space(8.0);
            schema_form::show(ui, &fields, &mut app.state.palette.form);
            if let Some(e) = &app.state.palette.form_error {
                ui.colored_label(t.danger, e);
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if widgets::primary_icon(ui, Some("play"), "Run", true).clicked() {
                    run = true;
                }
                if widgets::secondary(ui, None, "Back").clicked() {
                    back = true;
                }
            });
        });
    if !open {
        app.state.palette.open = false;
        app.state.palette.form_tool = None;
    }
    if back {
        app.state.palette.form_tool = None;
    }
    if run {
        match schema_form::build(&fields, &app.state.palette.form) {
            Ok(params) => {
                app.state.palette.open = false;
                app.state.palette.form_tool = None;
                let consent = if spec.effects.needs_consent() { Consent::Granted } else { Consent::NotAsked };
                app.run_tool(spec.id, params, Route::Palette, consent);
                app.set_mode(Mode::Experienced);
            }
            Err(e) => app.state.palette.form_error = Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn fuzzy_scores_prefer_prefix_and_contiguous() {
        assert!(score("Open a file", "open").unwrap() > score("Session status open", "open").unwrap());
        assert!(score("native_sections_list", "lst").is_some());
        assert!(score("native_sections_list", "xyz").is_none());
        assert_eq!(score("anything", ""), Some(0));
    }
}
