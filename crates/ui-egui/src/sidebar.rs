//! The left sidebar on the Simple screens. A header row of round buttons (search, new project,
//! fold), pinned builds (`crate::pins`), your projects, and a footer with the app's round mark
//! (Home) and Settings. A lifted surface with a hairline on its right edge, a quiet section
//! caption, and rounded two-line rows that take a soft fill on hover and when open.
//!
//! On macOS the sidebar runs to the top of the window and the system's window buttons sit over
//! its header, so the header leaves room for them and moves the window when dragged.

use crate::i18n::tr;
use crate::make::{ago, status_shown};
use crate::state::Screen;
use crate::theme::{self, Tokens};
use crate::titlebar::{NATIVE_CONTROLS, WINDOW_BUTTONS};
use crate::{CraftApp, icons, widgets};
use egui::{Color32, RichText, Sense, Ui, Vec2};

pub const WIDTH: f32 = 266.0;
/// Folded to an icon rail (wide enough for the system's window buttons on macOS).
pub const FOLDED: f32 = 76.0;
/// Below this window width the sidebar folds by itself, leaving the room to the screen.
const FOLD_BELOW: f32 = 900.0;
/// The side padding the panel gives the sidebar (`CraftApp::show`).
pub const PAD: f32 = 10.0;

/// Folded: by the person, or because the window is narrow.
pub fn folded(app: &CraftApp, ctx: &egui::Context) -> bool {
    app.state.sidebar_folded || ctx.content_rect().width() < FOLD_BELOW
}

pub fn width(app: &CraftApp, ctx: &egui::Context) -> f32 {
    if folded(app, ctx) { FOLDED } else { WIDTH }
}
/// The header row with the round buttons.
const HEADER: f32 = 54.0;
/// Round header and footer buttons.
const CIRCLE: f32 = 32.0;
/// A row with a second line (a project's status), and one without.
const ROW_H: f32 = 52.0;
const ROW_H_SINGLE: f32 = 40.0;
const AVATAR: f32 = 30.0;
/// Space between rows.
const ROW_GAP: f32 = 4.0;
/// The footer: the Settings pill and its padding.
const PILL_H: f32 = 34.0;
const FOOTER_BOTTOM: f32 = 14.0;

pub fn show(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let fold = folded(app, ui.ctx());
    let narrow = ui.ctx().content_rect().width() < FOLD_BELOW;
    ui.data_mut(|d| d.insert_temp(egui::Id::new("sidebar-folded"), fold));
    let w = ui.available_width();
    // Folded, the round buttons stack at the bottom of the rail with Settings; the header keeps
    // its height, so nothing above the projects moves when the sidebar folds.
    let fold_in_footer = fold && NATIVE_CONTROLS && !narrow;
    let footer_h = if fold {
        let n = 3.0 + f32::from(u8::from(fold_in_footer));
        n * (CIRCLE + 8.0) + FOOTER_BOTTOM
    } else {
        10.0 + PILL_H + FOOTER_BOTTOM
    };
    let top_h = (ui.available_height() - footer_h).max(120.0);
    ui.allocate_ui_with_layout(Vec2::new(w, top_h), egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_height(top_h);
        header(app, ui, fold, narrow);
        // Advanced mode's workbench sits beside the projects, so builds stay in sight.
        if !app.beginner() && row(ui, "microscope", "Workbench", None, false).clicked() {
            app.state.screen = Screen::Workbench;
            app.ensure_rail_loaded();
        }
        section_gap(ui, fold, 10.0);
        // Pinned builds, between the screens and the projects, to open in one click.
        crate::pins::section(app, ui, fold);
        // Folded, the heading's words don't fit: the icon stays, on the same line, so nothing
        // moves when the sidebar folds.
        let refresh = |ui: &mut Ui| icons::button(ui, "refresh-cw", 22.0, false, "Look for projects again").clicked();
        let again = if fold {
            centred_row(ui, 22.0, refresh)
        } else {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                widgets::caption(ui, "Projects");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    refresh(ui)
                })
                .inner
            })
            .inner
        };
        if again {
            app.refresh_projects();
        }
        ui.add_space(6.0);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = ROW_GAP;
            if app.projects.is_empty() && !fold {
                egui::Frame::new().inner_margin(egui::Margin { left: 8, right: 8, top: 2, bottom: 2 }).show(ui, |ui| {
                    ui.label(RichText::new(tr("Nothing made yet. What you make shows up here.")).color(t.text_faint).size(theme::CAPTION + 0.5));
                });
                return;
            }
            let open_root = (app.state.screen == Screen::Project).then(|| app.workspace.as_ref().map(|w| w.root.clone())).flatten();
            let mut open = None;
            for ws in app.projects.clone().iter() {
                let m = &ws.manifest;
                let (label, fg, _) = status_shown(app, ws, &t);
                // What needs the person is said in words, and in amber.
                let urgent = matches!(label, "Needs you" | "Interrupted");
                let when = ago(m.updated_ms);
                let r = row_with(ui, crate::make::project_icon(m), &m.name, Some((fg, label, urgent)), Some(&when), open_root.as_ref() == Some(&ws.root))
                    .on_hover_text(format!("{} · {when}", m.request));
                // Right-click: rename, duplicate or delete it.
                r.context_menu(|ui| crate::manage::menu(app, ui, ws));
                if r.clicked() {
                    open = Some(ws.clone());
                }
            }
            if let Some(ws) = open {
                app.open_workspace(ws);
            }
        });
    });
    footer(app, ui, fold, fold_in_footer);
}

/// The header row: room for the system's window buttons (macOS), then round buttons at the right:
/// search, New project and fold. Its empty part moves the window on macOS.
fn header(app: &mut CraftApp, ui: &mut Ui, fold: bool, narrow: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), HEADER), Sense::hover());
    if NATIVE_CONTROLS {
        // The panel's padding too, so the whole top of the sidebar moves the window.
        crate::titlebar::drag_region(ui, rect.expand2(Vec2::new(PAD, 0.0)), ui.id().with("sidebar-drag"));
    }
    if fold {
        // macOS: the window buttons fill the folded header; the fold button goes to the footer.
        if !NATIVE_CONTROLS && !narrow {
            let c = egui::Rect::from_center_size(rect.center(), Vec2::splat(CIRCLE));
            if circle_at(ui, c, "chevron-right", "Show the sidebar", false).clicked() {
                app.state.sidebar_folded = false;
            }
        }
        return;
    }
    // Right to left from the panel's edge (14 from the divider).
    let mut x = rect.right() - 4.0;
    let mut next = |ui: &mut Ui, icon: &str, tip: &str, on: bool| {
        let c = egui::Rect::from_min_size(egui::pos2(x - CIRCLE, rect.center().y - CIRCLE / 2.0), Vec2::splat(CIRCLE));
        x -= CIRCLE + 6.0;
        // Never over the window buttons, however narrow.
        if NATIVE_CONTROLS && c.left() < rect.left() - PAD + WINDOW_BUTTONS {
            return None;
        }
        Some(circle_at(ui, c, icon, tip, on))
    };
    if !narrow && next(ui, "chevron-left", "Fold the sidebar", false).is_some_and(|r| r.clicked()) {
        app.state.sidebar_folded = true;
    }
    if next(ui, "plus", "New project", app.state.screen == Screen::Home).is_some_and(|r| r.clicked()) {
        app.state.screen = Screen::Home;
    }
    if next(ui, "search", "Search", app.state.palette.open).is_some_and(|r| r.clicked()) {
        open_palette(app);
    }
}

/// The command palette, as Ctrl/⌘+K opens it.
fn open_palette(app: &mut CraftApp) {
    app.state.palette.open = true;
    app.state.palette.query.clear();
    app.state.palette.selected = 0;
    app.state.palette.form_tool = None;
}

/// Pinned to the bottom. Open: the app's round mark (Home) and an outlined Settings pill.
/// Folded: the round buttons the header can't hold, then Settings, stacked.
fn footer(app: &mut CraftApp, ui: &mut Ui, fold: bool, fold_in_footer: bool) {
    let bottom = ui.max_rect().bottom() - FOOTER_BOTTOM;
    let left = ui.max_rect().left();
    let w = ui.max_rect().width();
    if fold {
        let cx = ui.max_rect().center().x;
        let mut y = bottom - CIRCLE;
        let mut next = |ui: &mut Ui, icon: &str, tip: &str, on: bool| {
            let c = egui::Rect::from_min_size(egui::pos2(cx - CIRCLE / 2.0, y), Vec2::splat(CIRCLE));
            y -= CIRCLE + 8.0;
            circle_at(ui, c, icon, tip, on)
        };
        if next(ui, "settings", "Settings", app.state.settings_open).clicked() {
            app.state.settings_open = !app.state.settings_open;
        }
        if next(ui, "plus", "New project", app.state.screen == Screen::Home).clicked() {
            app.state.screen = Screen::Home;
        }
        if next(ui, "search", "Search", app.state.palette.open).clicked() {
            open_palette(app);
        }
        if fold_in_footer && next(ui, "chevron-right", "Show the sidebar", false).clicked() {
            app.state.sidebar_folded = false;
        }
        return;
    }
    let t = Tokens::get(ui.ctx());
    // The app's mark, round, where an account picture would be: it goes Home.
    let mark = egui::Rect::from_min_size(egui::pos2(left + 4.0, bottom - PILL_H + 1.0), Vec2::splat(CIRCLE));
    let resp = ui.interact(mark, ui.id().with("home-mark"), Sense::click());
    let h = widgets::hover_t(ui, &resp);
    ui.painter().circle_filled(mark.center(), CIRCLE / 2.0, t.secondary_fill);
    crate::app_icon(CIRCLE - 4.0).corner_radius(((CIRCLE - 4.0) / 2.0) as u8).paint_at(ui, mark.shrink(2.0));
    ui.painter().circle_stroke(mark.center(), CIRCLE / 2.0 - 0.5, egui::Stroke::new(1.0, theme::mix(t.chrome_stroke, t.text.gamma_multiply(0.3), h)));
    widgets::focus_ring(ui, &resp, mark);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Home")));
    if resp.on_hover_text(tr("Home")).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        app.state.screen = Screen::Home;
    }
    // Settings: an outlined pill filling the rest of the row.
    let pill = egui::Rect::from_min_max(egui::pos2(mark.right() + 8.0, bottom - PILL_H), egui::pos2(left + w - 4.0, bottom));
    let resp = ui.interact(pill, ui.id().with("settings-pill"), Sense::click());
    let h = widgets::hover_t(ui, &resp);
    let on = app.state.settings_open;
    let fill = if on { t.selected_row } else { t.text.gamma_multiply(0.05 * h) };
    ui.painter().rect(pill, t.r_pill(), fill, egui::Stroke::new(1.0, t.chrome_stroke), egui::StrokeKind::Inside);
    let g = ui.painter().layout_no_wrap(tr("Settings"), theme::medium(13.5), t.text);
    let total = 16.0 + 8.0 + g.size().x;
    let x0 = pill.center().x - total / 2.0;
    icons::paint(ui, egui::Rect::from_center_size(egui::pos2(x0 + 8.0, pill.center().y), Vec2::splat(16.0)), "settings", 16.0, t.text);
    ui.painter().galley(egui::pos2(x0 + 24.0, pill.center().y - g.size().y / 2.0), g, t.text);
    widgets::focus_ring(ui, &resp, pill);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, on, tr("Settings")));
    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        app.state.settings_open = !app.state.settings_open;
    }
}

/// A round button with a hairline ring at `rect`: the glyph in the text colour, a faint tint on
/// hover, the selected-row fill when `on`.
fn circle_at(ui: &mut Ui, rect: egui::Rect, icon: &str, tip: &str, on: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let resp = ui.interact(rect, ui.id().with(("sidebar-circle", tip)), Sense::click());
    let h = widgets::hover_t(ui, &resp);
    let r = rect.width() / 2.0;
    if on {
        ui.painter().circle_filled(rect.center(), r, t.selected_row);
    }
    ui.painter().circle_filled(rect.center(), r, t.text.gamma_multiply(0.05 * h));
    ui.painter().circle_stroke(rect.center(), r - 0.5, egui::Stroke::new(1.0, t.chrome_stroke));
    icons::paint(ui, rect, icon, 16.0, t.text);
    if resp.has_focus() {
        ui.painter().circle_stroke(rect.center(), r + 2.0, egui::Stroke::new(2.0, t.accent));
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(tip)));
    resp.on_hover_text(tr(tip)).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A folded section's heading: a row as tall as the open one, with something `width` wide centred.
pub(crate) fn centred_row<R>(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        add(ui)
    })
    .inner
}

/// The space before a section. Folded, a short rule across it takes the place of the section's name.
pub(crate) fn section_gap(ui: &mut Ui, fold: bool, height: f32) {
    let y = ui.cursor().top() + height / 2.0;
    ui.add_space(height);
    if fold {
        let t = Tokens::get(ui.ctx());
        ui.painter().hline(ui.max_rect().shrink2(Vec2::new(14.0, 0.0)).x_range(), y, egui::Stroke::new(1.0, t.separator));
    }
}

/// One sidebar row: a round avatar with the icon, the label, and for projects a second line with
/// the status (amber, with a dot on the avatar, when it needs the person: `urgent`). `on` = the
/// open one.
pub(crate) fn row(ui: &mut Ui, icon: &str, label: &str, status: Option<(Color32, &str, bool)>, on: bool) -> egui::Response {
    row_with(ui, icon, label, status, None, on)
}

/// [`row`] with a quiet note after the status on the second line (when it last changed).
fn row_with(ui: &mut Ui, icon: &str, label: &str, status: Option<(Color32, &str, bool)>, note: Option<&str>, on: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let fold = ui.data(|d| d.get_temp::<bool>(egui::Id::new("sidebar-folded")).unwrap_or(false));
    let height = if status.is_some() { ROW_H } else { ROW_H_SINGLE };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let h = widgets::hover_t(ui, &resp);
    // The row's fill, and the same colour laid on the sidebar for the ring around the status dot.
    let (fill, under) = if on { (t.selected_row, t.selected_row) } else { (theme::mix(Color32::TRANSPARENT, t.hover, h), theme::mix(t.chrome, t.hover, h)) };
    ui.painter().rect_filled(rect, t.r(), fill);
    // Project names are shown as written; the app's own rows are translated.
    let text = if status.is_some() { label.to_string() } else { tr(label) };
    let avatar_x = if fold { rect.center().x } else { rect.left() + 8.0 + AVATAR / 2.0 };
    let avatar = egui::pos2(avatar_x, rect.center().y);
    ui.painter().circle_filled(avatar, AVATAR / 2.0, if on { theme::mix(t.secondary_fill, t.text, 0.08) } else { t.secondary_fill });
    icons::paint(ui, egui::Rect::from_center_size(avatar, Vec2::splat(15.0)), icon, 15.0, t.text);
    if let Some((color, _, true)) = status {
        // Bottom-right of the avatar, cut out of it by a ring in the row's colour.
        let r = AVATAR * 0.18;
        let c = avatar + Vec2::splat(AVATAR / 2.0 * std::f32::consts::FRAC_1_SQRT_2);
        ui.painter().circle_filled(c, r + 2.0, under);
        ui.painter().circle_filled(c, r, color);
    }
    let name = match status {
        Some((_, s, _)) => format!("{text}, {}", tr(s)),
        None => text.clone(),
    };
    widgets::focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, on, &name));
    // Folded: the avatar (and its dot), with the name on hover.
    if fold {
        return resp.on_hover_text(name).on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    let text_x = rect.left() + 8.0 + AVATAR + 10.0;
    let max_w = (rect.right() - 10.0 - text_x).max(20.0);
    let line = |text: &str, font: egui::FontId, color: Color32| {
        let mut job = egui::text::LayoutJob::simple_singleline(text.to_string(), font, color);
        job.wrap = egui::text::TextWrapping::truncate_at_width(max_w);
        ui.painter().layout_job(job)
    };
    let title = line(&text, theme::medium(13.5), t.text);
    match status {
        Some((color, s, urgent)) => {
            // The status and when, as one quiet line; amber when it needs the person.
            let mut job = egui::text::LayoutJob::default();
            let fmt = |c: Color32| egui::TextFormat { font_id: egui::FontId::proportional(12.5), color: c, ..Default::default() };
            let shown = tr(s);
            job.append(&shown, 0.0, fmt(if urgent { color } else { t.text_dim }));
            if let Some(n) = note {
                job.append(&format!(" · {n}"), 0.0, fmt(t.text_dim));
            }
            job.wrap = egui::text::TextWrapping::truncate_at_width(max_w);
            let sub = ui.painter().layout_job(job);
            let gap = 2.0;
            let top = rect.center().y - (title.size().y + gap + sub.size().y) / 2.0;
            let sub_top = top + title.size().y + gap;
            ui.painter().galley(egui::pos2(text_x, top), title, t.text);
            ui.painter().galley(egui::pos2(text_x, sub_top), sub, t.text_dim);
        }
        None => {
            ui.painter().galley(egui::pos2(text_x, rect.center().y - title.size().y / 2.0), title, t.text);
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}
