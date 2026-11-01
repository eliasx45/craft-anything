//! The prompt box used everywhere the person types to the builder or the assistant: it grows
//! with the text (Enter sends, Shift+Enter adds a line), shows attached pictures as chips, and
//! has round attach, stop and send buttons beside the text. Up at the start of the box takes back
//! the last message still waiting, to edit it; with none waiting, Up at the very start and Down at
//! the very end go through earlier prompts, and Down past the newest brings back what was being
//! typed. Anywhere else they move the cursor as usual.

use crate::i18n::{tr, trf};
use crate::theme::{self, Tokens};
use crate::{icons, widgets};
use egui::{Margin, RichText, Ui};
use std::path::PathBuf;

/// How one prompt box looks and behaves.
pub struct PromptBox<'a> {
    /// The accessible name, and the id of the text field.
    pub name: &'a str,
    pub hint: String,
    /// Visible text rows: (at least, at most before it scrolls).
    pub rows: (usize, usize),
    /// The paperclip's tooltip; `None` hides it (no attachments here).
    pub attach: Option<&'a str>,
    /// A run or answer is in progress: a stop button beside Send, named this ("Stop", "Pause").
    pub stop_button: Option<&'a str>,
    /// Show the send button (the Make screen has its own big Start button).
    pub send: bool,
    /// A sent message is still waiting and can be taken back: Up at the start of the box asks for it.
    pub recall: bool,
    /// Earlier prompts, oldest first, for Up and Down; read only when a key asks for them.
    pub history: Option<&'a dyn Fn() -> Vec<String>>,
    pub font_size: f32,
}

/// Going through earlier prompts: how far back, and what the box holds for it to still count.
#[derive(Debug, Clone, Default, PartialEq)]
struct Browsing {
    back: usize,
    shown: String,
    /// What was being typed, for Down past the newest.
    draft: String,
}

/// The prompt before the one showing; none when there's no earlier one.
fn older(history: &[String], now: Option<&Browsing>, text: &str) -> Option<Browsing> {
    let back = now.map_or(1, |b| b.back + 1);
    let shown = history.get(history.len().checked_sub(back)?)?.clone();
    Some(Browsing { back, shown, draft: now.map_or_else(|| text.to_string(), |b| b.draft.clone()) })
}

/// The prompt after the one showing, or what was being typed once past the newest.
fn newer(history: &[String], now: &Browsing) -> Result<Browsing, String> {
    let back = now.back.saturating_sub(1);
    match history.len().checked_sub(back).and_then(|i| history.get(i)) {
        Some(shown) if back > 0 => Ok(Browsing { back, shown: shown.clone(), draft: now.draft.clone() }),
        _ => Err(now.draft.clone()),
    }
}

/// Up at the start or Down at the end went through earlier prompts: consumed before the field
/// sees them. Up leaves the cursor at the start and Down at the end, so pressing it again goes on.
fn browse(ui: &mut Ui, id: egui::Id, text: &mut String, history: &dyn Fn() -> Vec<String>) {
    let key = id.with("history");
    // Editing what came back, or sending it, ends going through them.
    let now = ui.data_mut(|d| d.get_temp::<Browsing>(key)).filter(|b| b.shown == *text);
    let list = || {
        let mut list = history();
        list.dedup();
        list
    };
    let plain = egui::Modifiers::NONE;
    let up = at_start(ui.ctx(), id, text) && ui.input_mut(|i| i.consume_key(plain, egui::Key::ArrowUp));
    let next = if up {
        older(&list(), now.as_ref(), text).map(Ok)
    } else {
        now.as_ref().filter(|_| at_end(ui.ctx(), id, text) && ui.input_mut(|i| i.consume_key(plain, egui::Key::ArrowDown))).map(|b| newer(&list(), b))
    };
    match next {
        Some(Ok(b)) => {
            text.clone_from(&b.shown);
            ui.data_mut(|d| d.insert_temp(key, b));
        }
        Some(Err(draft)) => {
            *text = draft;
            ui.data_mut(|d| d.remove_temp::<Browsing>(key));
        }
        None => return,
    }
    if up {
        cursor_to(ui.ctx(), id, 0);
    } else {
        cursor_to_end(ui.ctx(), id, text);
    }
}

/// What the person did with it this frame.
#[derive(Default)]
pub struct PromptOut {
    /// Send was pressed (button or Enter) with something to send.
    pub send: bool,
    pub attach: bool,
    pub stop: bool,
    /// Up was pressed at the start of the box while a message could be taken back.
    pub recall: bool,
    /// The attachment to remove.
    pub remove: Option<usize>,
}

pub fn show(ui: &mut Ui, text: &mut String, attachments: &[PathBuf], b: PromptBox) -> PromptOut {
    let t = Tokens::get(ui.ctx());
    let mut out = PromptOut::default();
    let id = ui.make_persistent_id(("prompt-box", b.name));
    let focused = ui.memory(|m| m.has_focus(id));
    // A floating surface: its own fill, a faint outline (a little stronger while typing) and a
    // soft shadow.
    let outline = crate::chat::faint_overlay(&t, if focused { 0.26 } else { 0.18 });
    let frame = egui::Frame::new()
        .fill(t.composer)
        .stroke(egui::Stroke::new(1.0, outline))
        .corner_radius(t.r_xl())
        .shadow(egui::Shadow { offset: [0, 2], blur: 20, spread: 0, color: egui::Color32::from_black_alpha(if t.dark() { 70 } else { 18 }) })
        .inner_margin(Margin { left: 8, right: 8, top: 7, bottom: 7 });
    // Stacked top to bottom whatever the caller's layout (it's often placed in a row).
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        frame.show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_width(ui.available_width());
                if !attachments.is_empty() {
                    attachment_chips(ui, attachments, &t, &mut out);
                    ui.add_space(6.0);
                }
                // Attach on the left, the text in the middle, stop and send on the right, all
                // sitting on the last line as the text grows.
                use crate::chat::{Glyph, Round, round_button};
                // The text is laid out first (its height decides the row's), then the buttons
                // are placed level with its last line.
                let (gap, row) = (8.0, ui.available_rect_before_wrap());
                // A box that is several lines tall from the start (the Make screen) keeps its text
                // the full width and puts the round controls on a row of their own underneath,
                // so no empty column runs beside the text.
                let stacked = b.rows.0 > 1;
                let left = if b.attach.is_some() && !stacked { CONTROL + gap } else { 0.0 };
                let buttons = usize::from(b.send) + usize::from(b.stop_button.is_some());
                let right = if stacked { 0.0 } else { buttons as f32 * (CONTROL + gap) };
                let field_rect = egui::Rect::from_min_max(
                    egui::pos2(row.left() + left, row.top()),
                    egui::pos2((row.right() - right).max(row.left() + left + 60.0), row.bottom()),
                );
                let mut field_ui = ui.new_child(egui::UiBuilder::new().max_rect(field_rect).layout(egui::Layout::top_down(egui::Align::Min)));
                field(&mut field_ui, id, text, &b, focused, &mut out);
                let text_h = field_ui.min_rect().height();
                let has_controls = b.attach.is_some() || buttons > 0;
                let h = if stacked && has_controls { text_h + 6.0 + CONTROL } else { text_h.max(CONTROL) };
                let at = |x: f32| egui::Rect::from_min_size(egui::pos2(x, row.top() + h - CONTROL), egui::Vec2::splat(CONTROL));
                let place = |ui: &mut Ui, x: f32, add: &mut dyn FnMut(&mut Ui) -> bool| {
                    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(at(x)).layout(egui::Layout::left_to_right(egui::Align::Min)));
                    add(&mut child)
                };
                if let Some(tip) = b.attach {
                    out.attach |= place(ui, row.left(), &mut |ui| round_button(ui, CONTROL, Round::Control, Glyph::Plus, tip).clicked());
                }
                let mut x = row.right() - CONTROL;
                if b.send {
                    let ready = !text.trim().is_empty();
                    out.send |= place(ui, x, &mut |ui| send_button(ui, ready).clicked() && ready);
                    x -= CONTROL + gap;
                }
                if let Some(name) = b.stop_button {
                    let glyph = if name == "Pause" { Glyph::Pause } else { Glyph::Stop };
                    out.stop |= place(ui, x, &mut |ui| round_button(ui, CONTROL, Round::Control, glyph, name).clicked());
                }
                ui.allocate_rect(egui::Rect::from_min_size(row.min, egui::vec2(row.width(), h)), egui::Sense::hover());
            })
        });
        // How the keys work, quietly under the box.
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(RichText::new(tr("Enter to send · Shift+Enter for a new line")).size(theme::TINY).color(t.text_faint));
        });
    });
    out
}

/// The round controls in the box (attach, stop, send).
const CONTROL: f32 = 25.0;

/// Attached pictures as small pills, each with its own remove button.
fn attachment_chips(ui: &mut Ui, attachments: &[PathBuf], t: &Tokens, out: &mut PromptOut) {
    ui.horizontal_wrapped(|ui| {
        for (i, p) in attachments.iter().enumerate() {
            let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            egui::Frame::new().fill(t.composer_control).corner_radius(t.r_pill()).inner_margin(Margin { left: 10, right: 2, top: 2, bottom: 2 }).show(
                ui,
                |ui| {
                    ui.horizontal(|ui| {
                        icons::show(ui, "paperclip", 13.0, t.icon);
                        ui.add(egui::Label::new(RichText::new(&name).size(theme::SMALL)).truncate()).on_hover_text(p.display().to_string());
                        if icons::button(ui, "x", 22.0, false, &trf("Remove {name}", &[("name", &name)])).clicked() {
                            out.remove = Some(i);
                        }
                    });
                },
            );
        }
    });
}

/// The text field: grows with its text up to `rows.1` lines, then scrolls.
fn field(ui: &mut Ui, id: egui::Id, text: &mut String, b: &PromptBox, focused: bool, out: &mut PromptOut) {
    let line = ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(b.font_size)));
    // Up at the very start (or in an empty box) takes back the last waiting message; it's
    // consumed before the field sees it, and anywhere else Up moves the cursor as usual.
    // The caller fills the text in after this frame; the cursor goes to its end on the next.
    let to_end = id.with("cursor-to-end");
    if ui.data_mut(|d| d.remove_temp::<bool>(to_end)).is_some() {
        cursor_to_end(ui.ctx(), id, text);
    }
    if b.recall && focused && at_start(ui.ctx(), id, text) && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
        out.recall = true;
        ui.data_mut(|d| d.insert_temp(to_end, true));
    } else if let Some(history) = b.history.filter(|_| focused && !b.recall) {
        browse(ui, id, text, history);
    }
    // One line sits level with the round buttons beside it.
    let pad_y = ((CONTROL - line) / 2.0).max(1.0);
    // No "more below" fade: the frame after each keystroke the scroll area briefly
    // measures a few points of overflow, and the fade blinked over the line being typed.
    ui.spacing_mut().scroll.fade.strength = 0.0;
    egui::ScrollArea::vertical().id_salt(id.with("scroll")).max_height(line * b.rows.1 as f32 + pad_y * 2.0 + 4.0).stick_to_bottom(true).show(ui, |ui| {
        let r = ui.add(
            egui::TextEdit::multiline(text)
                .id(id)
                .hint_text(RichText::new(&b.hint).color(Tokens::get(ui.ctx()).text_faint))
                .frame(egui::Frame::NONE.inner_margin(Margin::symmetric(4, pad_y.round() as i8)))
                .desired_rows(b.rows.0)
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(b.font_size))
                // Enter sends; Shift+Enter starts a new line.
                .return_key(Some(egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::Enter))),
        );
        widgets::named(r.clone(), b.name);
        if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) && !text.trim().is_empty() {
            out.send = true;
        }
    });
}

/// The cursor is at this character, selecting nothing (or there's no text, or no cursor yet).
fn cursor_at(ctx: &egui::Context, id: egui::Id, text: &str, index: usize) -> bool {
    text.is_empty()
        || egui::TextEdit::load_state(ctx, id).and_then(|s| s.cursor.char_range()).is_none_or(|r| r.primary.index.0 == index && r.secondary.index.0 == index)
}

fn at_start(ctx: &egui::Context, id: egui::Id, text: &str) -> bool {
    cursor_at(ctx, id, text, 0)
}

fn at_end(ctx: &egui::Context, id: egui::Id, text: &str) -> bool {
    cursor_at(ctx, id, text, text.chars().count())
}

fn cursor_to(ctx: &egui::Context, id: egui::Id, index: usize) {
    let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(index))));
    egui::TextEdit::store_state(ctx, id, state);
}

/// Put the cursor at the end of the text.
fn cursor_to_end(ctx: &egui::Context, id: egui::Id, text: &str) {
    cursor_to(ctx, id, text.chars().count());
}

/// Send: a white (black in light) circle with an arrow once there's something to send, a dim
/// one before.
fn send_button(ui: &mut Ui, ready: bool) -> egui::Response {
    use crate::chat::{Glyph, Round};
    crate::chat::round_button(ui, CONTROL, if ready { Round::Primary } else { Round::PrimaryDim }, Glyph::ArrowUp, "Send (Enter)")
}

/// "2 pictures attached" for the feed, or nothing.
pub fn attached_line(n: usize) -> String {
    match n {
        0 => String::new(),
        1 => tr("(1 picture attached)"),
        n => trf("({n} pictures attached)", &[("n", &n.to_string())]),
    }
}

/// A sent message as it was typed: what the feed shows, without its [`attached_line`].
pub fn as_typed(shown: &str) -> &str {
    let Some((text, last)) = shown.rsplit_once('\n') else { return shown };
    let n: Option<usize> = last.chars().filter(char::is_ascii_digit).collect::<String>().parse().ok();
    if n.is_some_and(|n| attached_line(n) == last) { text } else { shown }
}

/// The text the agent gets: the message, then where its pictures are so it opens them.
pub fn with_attachments(text: &str, files: &[PathBuf], root: &std::path::Path) -> String {
    if files.is_empty() {
        return text.to_string();
    }
    let list = files.iter().map(|p| p.strip_prefix(root).unwrap_or(p).display().to_string().replace('\\', "/")).collect::<Vec<_>>().join(", ");
    format!("{text}\n\nAttached pictures: {list}. Open each one with the Read tool; they show what the person means.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_goes_back_through_prompts_and_down_returns_to_what_was_being_typed() {
        let list = ["one".to_string(), "two".to_string()];
        let first = older(&list, None, "half a thou").unwrap();
        assert_eq!((first.shown.as_str(), first.draft.as_str()), ("two", "half a thou"));
        let second = older(&list, Some(&first), &first.shown).unwrap();
        assert_eq!((second.shown.as_str(), second.draft.as_str()), ("one", "half a thou"));
        assert_eq!(older(&list, Some(&second), &second.shown), None, "the oldest stays");
        assert_eq!(newer(&list, &second).unwrap().shown, "two");
        assert_eq!(newer(&list, &first), Err("half a thou".to_string()));
        assert_eq!(older(&[], None, ""), None);
    }

    #[test]
    fn a_prompt_comes_back_without_its_pictures_line() {
        assert_eq!(as_typed("Add tags\n(2 pictures attached)"), "Add tags");
        assert_eq!(as_typed("Add tags\n(1 picture attached)"), "Add tags");
        assert_eq!(as_typed("First\nsecond line"), "First\nsecond line");
        assert_eq!(as_typed("Use 3\ncolours"), "Use 3\ncolours");
    }
}
