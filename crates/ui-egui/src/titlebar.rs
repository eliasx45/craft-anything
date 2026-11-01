//! The window's top chrome. On Windows and Linux the native frame is off and our own title bar
//! draws the app mark, what's open, Clean room / Deep, AI backend, Simple · Advanced, settings and
//! the window buttons; empty parts drag the window and the window edges resize it. On macOS the
//! system's own window buttons sit over the sidebar (the content runs under a see-through title
//! bar), so there is no bar across the window: the main area's top strip drags the window and
//! carries a small pill with what's open and the Clean room switch.

use crate::i18n::tr;
use crate::state::Screen;
use crate::theme::{self, Tokens};
use crate::{CraftApp, icons, widgets};
use craft_anything_engine::Mode;
use egui::viewport::ResizeDirection;
use egui::{RichText, Ui, Vec2};

/// Height of the custom title bar.
pub const HEIGHT: f32 = 40.0;
/// How close to the window edge the pointer must be to resize.
const EDGE: f32 = 5.0;
/// The system draws the window buttons (macOS): the window keeps its native frame, edges and
/// shadow, and our content runs under a see-through title bar.
pub const NATIVE_CONTROLS: bool = cfg!(target_os = "macos");
/// Room the system's window buttons take at the top-left (macOS).
pub const WINDOW_BUTTONS: f32 = 80.0;
/// The draggable strip at the top of the main area when there is no title bar (macOS).
pub const MAIN_TOP: f32 = 40.0;

fn maximized(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().maximized.unwrap_or(false) || i.viewport().fullscreen.unwrap_or(false))
}

/// The window edge (or corner) under the pointer, when the window can be resized from there.
pub fn edge_at(ctx: &egui::Context) -> Option<ResizeDirection> {
    if NATIVE_CONTROLS || maximized(ctx) {
        return None;
    }
    let pos = ctx.input(|i| i.pointer.hover_pos())?;
    let r = ctx.content_rect();
    let (w, e, n, s) = (pos.x - r.left() < EDGE, r.right() - pos.x < EDGE, pos.y - r.top() < EDGE, r.bottom() - pos.y < EDGE);
    Some(match (w, e, n, s) {
        (true, _, true, _) => ResizeDirection::NorthWest,
        (_, true, true, _) => ResizeDirection::NorthEast,
        (true, _, _, true) => ResizeDirection::SouthWest,
        (_, true, _, true) => ResizeDirection::SouthEast,
        (true, ..) => ResizeDirection::West,
        (_, true, ..) => ResizeDirection::East,
        (_, _, true, _) => ResizeDirection::North,
        (_, _, _, true) => ResizeDirection::South,
        _ => return None,
    })
}

/// A 1 px edge around the window (the native frame is off), so it doesn't melt into the desktop.
pub fn window_edge(ctx: &egui::Context) {
    if NATIVE_CONTROLS || maximized(ctx) {
        return;
    }
    let t = Tokens::get(ctx);
    ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("window-edge"))).rect_stroke(
        ctx.content_rect(),
        0.0,
        egui::Stroke::new(1.0, t.field_border),
        egui::StrokeKind::Inside,
    );
}

/// Resizing from the window edges (the native frame is off, so we provide it).
pub fn resize_edges(ctx: &egui::Context) {
    let Some(dir) = edge_at(ctx) else { return };
    ctx.set_cursor_icon(match dir {
        ResizeDirection::North => egui::CursorIcon::ResizeNorth,
        ResizeDirection::South => egui::CursorIcon::ResizeSouth,
        ResizeDirection::East => egui::CursorIcon::ResizeEast,
        ResizeDirection::West => egui::CursorIcon::ResizeWest,
        ResizeDirection::NorthEast => egui::CursorIcon::ResizeNorthEast,
        ResizeDirection::NorthWest => egui::CursorIcon::ResizeNorthWest,
        ResizeDirection::SouthEast => egui::CursorIcon::ResizeSouthEast,
        ResizeDirection::SouthWest => egui::CursorIcon::ResizeSouthWest,
    });
    if ctx.input(|i| i.pointer.primary_pressed()) {
        ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
    }
}

/// A thin vertical rule between title bar cells.
fn rule(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(Vec2::new(1.0, HEIGHT), egui::Sense::hover());
    ui.painter().vline(r.center().x, r.y_range(), egui::Stroke::new(1.0, t.separator));
}

/// One of the window buttons (minimise, maximise, close): a full-height cell.
fn window_button(ui: &mut Ui, icon: &str, tip: &str, close: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(46.0, HEIGHT), egui::Sense::click());
    let hot = resp.hovered();
    if hot {
        ui.painter().rect_filled(rect, 0.0, if close { t.danger } else { t.hover });
    }
    let col = if hot && close { egui::Color32::WHITE } else { t.icon };
    icons::paint(ui, egui::Rect::from_center_size(rect.center(), Vec2::splat(14.0)), icon, 14.0, col);
    widgets::focus_ring(ui, &resp, rect.shrink(3.0));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(tip)));
    resp.on_hover_text(tr(tip))
}

/// Empty chrome that moves the window: drag to move, double-click to zoom. Register it before the
/// controls on top of it, so they keep their own clicks.
pub fn drag_region(ui: &mut Ui, rect: egui::Rect, id: egui::Id) {
    let ctx = ui.ctx().clone();
    let drag = ui.interact(rect, id, egui::Sense::click_and_drag());
    if edge_at(&ctx).is_none() && drag.drag_started_by(egui::PointerButton::Primary) {
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized(&ctx)));
    }
}

/// The title bar across the window: always on Windows and Linux; on macOS only where there is no
/// sidebar to hold the system's window buttons (first run, the workbench).
pub fn show(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let bar = ui.max_rect();
    drag_region(ui, bar, ui.id().with("titlebar-drag"));
    ui.painter().hline(bar.x_range(), bar.bottom() - 0.5, egui::Stroke::new(1.0, t.separator));
    // A download anywhere in the app fills the bottom edge of the bar, on every screen.
    let download = crate::engine_ui::global_download(app);
    if let Some((_, Some(f), _)) = &download {
        let x = bar.left() + bar.width() * f.clamp(0.0, 1.0);
        ui.painter().line_segment([egui::pos2(bar.left(), bar.bottom() - 1.0), egui::pos2(x, bar.bottom() - 1.0)], egui::Stroke::new(2.0, t.accent));
    }
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        // macOS: the system's window buttons sit at the left.
        ui.add_space(if NATIVE_CONTROLS { WINDOW_BUTTONS - 10.0 } else { 12.0 });
        if ui.add(crate::app_icon(22.0).corner_radius(11).sense(egui::Sense::click())).on_hover_text(tr("Home")).clicked() {
            app.state.screen = Screen::Home;
        }
        ui.label(theme::label_text("Craft Anything", 12.0).color(t.text));
        if let Some((project, target)) = breadcrumb(app) {
            rule(ui);
            if let Some(p) = project {
                ui.label(theme::label_text(&p, 12.0).color(t.text_dim));
                if target.is_some() {
                    ui.label(RichText::new("/").color(t.text_faint));
                }
            }
            if let Some((name, kind)) = target {
                icons::show(ui, crate::kind_icon(kind), 14.0, t.icon);
                ui.add(egui::Label::new(RichText::new(name).font(theme::mono(11.5)).color(t.text)).truncate()).on_hover_text(tr(kind.plain()));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if NATIVE_CONTROLS {
                ui.add_space(8.0);
            } else {
                if window_button(ui, "x", "Close", true).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                let max = maximized(&ctx);
                if window_button(ui, if max { "layers" } else { "square" }, if max { "Restore" } else { "Maximise" }, false).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
                }
                if window_button(ui, "minus", "Minimise", false).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
                rule(ui);
            }
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.add_space(4.0);
            if let Some((title, fraction, message)) = download {
                if download_pill(ui, &title, fraction).on_hover_text(message).clicked() {
                    open_download(app);
                }
                rule(ui);
            }
            // During first run there is nothing to switch or set yet.
            if !app.settings.first_run_done {
                return;
            }
            // Simple mode keeps the bar to the essentials: Settings is in the sidebar, the mode
            // switch and AI picker are in Settings, and the Clean room switch stays in view.
            if app.beginner() {
                deep_toggle(app, ui, true);
                usage_pill(app, ui);
                return;
            }
            if icons::button(ui, "settings", 30.0, app.state.settings_open, "Settings").clicked() {
                app.state.settings_open = !app.state.settings_open;
            }
            rule(ui);
            advanced_controls(app, ui, true);
        });
    });
}

/// macOS, beside the sidebar: the main area's top strip. It moves the window, and its right end
/// holds what's open and the Clean room switch in one pill, then what the AI has used, a
/// download in progress and, in Advanced mode, the mode switch and the AI picker.
pub fn main_top(app: &mut CraftApp, ui: &mut Ui) {
    let strip = ui.max_rect();
    drag_region(ui, strip, ui.id().with("main-top-drag"));
    let download = crate::engine_ui::global_download(app);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add_space(14.0);
        context_pill(app, ui);
        usage_pill(app, ui);
        if let Some((title, fraction, message)) = download
            && download_pill(ui, &title, fraction).on_hover_text(message).clicked()
        {
            open_download(app);
        }
        if !app.beginner() {
            advanced_controls(app, ui, false);
        }
    });
}

/// Advanced mode's controls, laid out right to left: Simple · Advanced and the AI picker (in the
/// title bar also the Clean room switch and what the AI has used, which the main area's strip
/// shows on its own), then a spinner while jobs run.
fn advanced_controls(app: &mut CraftApp, ui: &mut Ui, in_title_bar: bool) {
    let t = Tokens::get(ui.ctx());
    let sel = usize::from(app.mode() == Mode::Experienced);
    let switch_mode = |app: &mut CraftApp, ui: &mut Ui| {
        if let Some(i) = widgets::segmented(ui, &["Simple", "Advanced"], sel, 28.0) {
            app.set_mode(if i == 0 { Mode::Beginner } else { Mode::Experienced });
        }
    };
    if in_title_bar {
        switch_mode(app, ui);
        rule(ui);
        backend_picker(app, ui);
        deep_toggle(app, ui, true);
        usage_pill(app, ui);
    } else {
        backend_picker(app, ui);
        switch_mode(app, ui);
    }
    if app.jobs.busy() {
        let n = app.jobs.running().filter(|j| j.visible).count();
        ui.add(egui::Spinner::new().size(14.0).color(t.text_dim)).on_hover_text(crate::i18n::trf("{n} job(s) running", &[("n", &n.to_string())]));
    }
}

/// What's open, for the breadcrumb: the project (or the open target's project) and the target.
type Crumbs = (Option<String>, Option<(String, craft_anything_engine::TargetKind)>);
fn breadcrumb(app: &CraftApp) -> Option<Crumbs> {
    let in_project = app.state.screen == Screen::Project;
    let project = if in_project { app.workspace.as_ref().map(|w| w.manifest.name.clone()) } else { app.view.project.clone() };
    let target = if in_project { None } else { app.view.active_target().map(|t| (t.name.clone(), t.kind)) };
    (project.is_some() || target.is_some()).then_some((project, target))
}

/// The engine's steps for a download in progress.
fn open_download(app: &mut CraftApp) {
    app.state.modal = Some(match app.engine_flow.job {
        Some((b, _)) => crate::state::Modal::Engine(b),
        None => crate::state::Modal::Ghidra,
    });
}

/// A pill at the top-right of the main area: what's open ("Snake", or "project / file"), a
/// hairline, then the Clean room / Deep switch. With nothing open it is the switch alone.
fn context_pill(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    const H: f32 = 32.0;
    // The Project screen names the project in its own title pill; saying it twice is noise.
    let crumbs = breadcrumb(app).filter(|_| app.state.screen != Screen::Project).map(|(project, target)| {
        let text = match (&project, &target) {
            (Some(p), Some((n, _))) => format!("{p} / {n}"),
            (Some(p), None) => p.clone(),
            (None, Some((n, _))) => n.clone(),
            (None, None) => String::new(),
        };
        let mut job = egui::text::LayoutJob::simple_singleline(text.clone(), theme::medium(13.0), t.text_dim);
        job.wrap = egui::text::TextWrapping::truncate_at_width(240.0);
        (ui.painter().layout_job(job), text, target.map(|(_, k)| k))
    });
    let toggle_w = deep_toggle_width(app, ui);
    let lead = crumbs.as_ref().map_or(0.0, |(g, _, kind)| 14.0 + kind.map_or(0.0, |_| 20.0) + g.size().x + 10.0);
    let size = Vec2::new(lead + toggle_w, H);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect(rect, t.r_pill(), t.pill_fill, egui::Stroke::new(1.0, t.chrome_stroke), egui::StrokeKind::Inside);
    if let Some((g, text, kind)) = crumbs {
        let mut x = rect.left() + 14.0;
        if let Some(k) = kind {
            icons::paint(ui, egui::Rect::from_center_size(egui::pos2(x + 7.0, rect.center().y), Vec2::splat(14.0)), crate::kind_icon(k), 14.0, t.icon);
            x += 20.0;
        }
        let text_rect = egui::Rect::from_min_size(egui::pos2(x, rect.center().y - g.size().y / 2.0), g.size());
        ui.painter().galley(text_rect.min, g, t.text_dim);
        let r = ui.interact(text_rect, ui.id().with("breadcrumb"), egui::Sense::hover());
        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &text));
        r.on_hover_text(&text);
        let sep_x = rect.left() + lead - 0.5;
        ui.painter().vline(sep_x, rect.shrink2(Vec2::new(0.0, 9.0)).y_range(), egui::Stroke::new(1.0, t.chrome_stroke));
    }
    let toggle_rect = egui::Rect::from_min_max(egui::pos2(rect.right() - toggle_w, rect.top()), rect.max);
    // A child that doesn't move this layout's cursor: the pill already took its room.
    let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(toggle_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
    deep_toggle(app, &mut inner, false);
}

/// What the AI has used since the app opened ("about $3.20", or tokens without a price), once
/// it has used anything; the details are in its tooltip.
fn usage_pill(app: &CraftApp, ui: &mut Ui) {
    let u = &app.session_usage;
    if !u.used_anything() {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let text = if u.priced { crate::usage::cost_short(u) } else { crate::usage::tokens_words(u.tokens()) };
    let g = widgets::label_galley(ui, &text, 11.0, t.text_dim);
    let (rect, r) = ui.allocate_exact_size(Vec2::new(g.size().x + 24.0, 32.0), egui::Sense::hover());
    ui.painter().rect(rect, t.r_pill(), t.pill_fill, egui::Stroke::new(1.0, t.chrome_stroke), egui::StrokeKind::Inside);
    ui.painter().galley_with_override_text_color(rect.center() - g.size() / 2.0, g, t.text_dim);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &text));
    r.on_hover_ui(|ui| {
        ui.label(RichText::new(tr("AI use this session")).font(theme::semibold(theme::SMALL)));
        ui.label(crate::usage::explain(u, "Since Craft Anything opened."));
    });
}

/// "Getting Godot · 42%" with a small bar; opens the engine's steps when clicked.
fn download_pill(ui: &mut Ui, title: &str, fraction: Option<f32>) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let text = match fraction {
        Some(f) => format!("{title} · {:.0}%", f * 100.0),
        None => title.to_string(),
    };
    let g = widgets::label_galley(ui, &text, 12.0, t.text);
    let size = Vec2::new(g.size().x + 32.0 + 56.0, 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let h = widgets::hover_t(ui, &resp);
    ui.painter().rect(rect, t.r_pill(), t.pill_fill, egui::Stroke::new(1.0, t.chrome_stroke), egui::StrokeKind::Inside);
    ui.painter().rect_filled(rect, t.r_pill(), t.text.gamma_multiply(0.05 * h));
    icons::paint(ui, egui::Rect::from_center_size(egui::pos2(rect.left() + 18.0, rect.center().y), Vec2::splat(14.0)), "download", 14.0, t.text);
    ui.painter().galley_with_override_text_color(egui::pos2(rect.left() + 32.0, rect.center().y - g.size().y / 2.0), g, t.text);
    let track = egui::Rect::from_min_size(egui::pos2(rect.right() - 50.0, rect.center().y - 1.5), Vec2::new(38.0, 3.0));
    ui.painter().rect_filled(track, 1.5, t.separator);
    let f = fraction.unwrap_or_else(|| (ui.input(|i| i.time) * 0.6).fract() as f32).clamp(0.0, 1.0);
    ui.painter().rect_filled(egui::Rect::from_min_size(track.min, Vec2::new(track.width() * f, track.height())), 1.5, t.accent);
    widgets::focus_ring(ui, &resp, rect);
    widgets::animate(ui, std::time::Duration::from_millis(250));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text.clone()));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// How wide the Clean room / Deep switch is: icon, word and the little switch.
fn deep_toggle_width(app: &CraftApp, ui: &Ui) -> f32 {
    let t = Tokens::get(ui.ctx());
    let label = tr(if app.settings.deep_mode { "Deep" } else { "Clean room" });
    widgets::label_galley(ui, &label, 11.5, t.text).size().x + 88.0
}

/// Clean room / Deep switch: an icon, the mode's name and a small switch, 32 tall. `ringed`
/// draws its own hairline pill (the title bar); inside the main area's pill it has none.
fn deep_toggle(app: &mut CraftApp, ui: &mut Ui, ringed: bool) {
    let t = Tokens::get(ui.ctx());
    let deep = app.settings.deep_mode;
    let missing = app.ghidra_missing();
    let label = tr(if deep { "Deep" } else { "Clean room" });
    let g = widgets::label_galley(ui, &label, 11.5, t.text);
    let size = Vec2::new(deep_toggle_width(app, ui), 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let h = widgets::hover_t(ui, &resp);
    let fill = t.text.gamma_multiply(0.06 * h);
    let stroke = if ringed { egui::Stroke::new(1.0, t.chrome_stroke) } else { egui::Stroke::NONE };
    ui.painter().rect(rect, t.r_pill(), fill, stroke, egui::StrokeKind::Inside);
    let icon_rect = egui::Rect::from_center_size(egui::pos2(rect.left() + 22.0, rect.center().y), Vec2::splat(16.0));
    let icon = if deep { "microscope" } else { "shield-check" };
    icons::paint(ui, icon_rect, icon, 15.0, theme::mix(t.icon, t.text, h));
    ui.painter().galley_with_override_text_color(egui::pos2(rect.left() + 36.0, rect.center().y - g.size().y / 2.0), g, t.text);
    // The little switch: grey when off, the accent with an inverted knob when on.
    let sw = egui::Rect::from_center_size(egui::pos2(rect.right() - 25.0, rect.center().y), Vec2::new(26.0, 16.0));
    let how = ui.ctx().animate_bool_with_time(resp.id, deep, 0.12);
    ui.painter().rect_filled(sw, t.r_pill(), theme::mix(t.secondary_fill, t.accent, how));
    let knob_x = egui::lerp(sw.left() + 8.0..=sw.right() - 8.0, how);
    let knob = theme::mix(egui::Color32::WHITE, t.on_accent, how);
    ui.painter().circle_filled(egui::pos2(knob_x, sw.center().y), 6.0, knob);
    widgets::focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, !deep, tr("Clean room")));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_ui(|ui| {
        ui.label(RichText::new(tr("Clean room or Deep")).font(theme::semibold(theme::SMALL)));
        ui.label(tr(
            "Clean room: nothing inside the original is read. We work from what you can see: pictures, the page as a visitor sees it, and watching it run.",
        ));
        ui.label(tr("Deep: we read its code, files and page source with every tool we have, and with Ghidra when it's installed. Click to switch."));
        if missing {
            ui.label(RichText::new(tr("Ghidra isn't installed, so the built-in decompiler stands in. Get it in Settings → Engines.")).color(t.warning));
        }
    });
    if resp.clicked() {
        app.set_deep(!deep);
    }
}

/// AI backend picker: configured brains, "No AI", "Set up…".
fn backend_picker(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let current = format!("{} · {}", tr("AI"), app.state.backend.clone().unwrap_or_else(|| tr("None")));
    let mut choice: Option<Option<String>> = None;
    let mut setup = false;
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 30.0;
        egui::ComboBox::from_id_salt("backend-picker")
            .selected_text(theme::label_text(&current, 12.0).color(if app.state.backend.is_some() { t.text } else { t.text_dim }))
            .icon(widgets::combo_icon)
            .width(190.0)
            .show_ui(ui, |ui| {
                for b in &app.backends {
                    let on = app.state.backend.as_ref() == Some(&b.name);
                    if ui.selectable_label(on, &b.name).on_hover_text(format!("{:?} · {}", b.kind, b.model)).clicked() {
                        choice = Some(Some(b.name.clone()));
                    }
                }
                if ui.selectable_label(app.state.backend.is_none(), tr("No AI (built-in engines only)")).clicked() {
                    choice = Some(None);
                }
                ui.separator();
                if ui.selectable_label(false, tr("Set up a brain…")).clicked() {
                    setup = true;
                }
            })
            .response
            .on_hover_text(tr("Which AI answers your questions"));
    });
    if let Some(c) = choice {
        app.choose_backend(c);
    }
    if setup {
        app.state.settings_open = true;
        app.state.settings_tab = crate::state::SettingsTab::Ai;
        app.detect_backends();
    }
}
