//! The conversation: turning agent events into chat items, and drawing them (Beginner Ask screen
//! and the Experienced agent console share the renderer).

use crate::i18n::tr;
use crate::state::{ChatItem, ConsentAnswer};
use crate::theme::{self, Tokens};
use crate::{CraftApp, evidence_ui, glossary, icons, widgets};
use craft_anything_ai::AgentEvent;
use craft_anything_engine::TargetKind;
use egui::{Color32, Margin, RichText, Stroke, Ui, Vec2};
use serde_json::Value;

/// Fold one agent event into the conversation.
pub fn apply_event(chat: &mut Vec<ChatItem>, e: AgentEvent) {
    match e {
        AgentEvent::TextDelta { text } => {
            if let Some(ChatItem::Assistant { text: t, streaming: true, .. }) = chat.last_mut() {
                t.push_str(&text);
            } else {
                // Status lines from this turn are superseded once the answer starts.
                if matches!(chat.last(), Some(ChatItem::Status(_))) {
                    chat.pop();
                }
                chat.push(ChatItem::Assistant { text, backend: None, streaming: true });
            }
        }
        // For logs, not the conversation.
        AgentEvent::Output { .. } => {}
        AgentEvent::Status { text } => {
            if let Some(ChatItem::Status(s)) = chat.last_mut() {
                *s = text;
            } else {
                chat.push(ChatItem::Status(text));
            }
        }
        AgentEvent::ToolStarted { call_id, tool, .. } => {
            end_streaming(chat);
            chat.push(ChatItem::Tool { call_id, tool, ok: None, summary: String::new(), evidence_id: None, evidence: None });
        }
        AgentEvent::ToolFinished { call_id, tool, ok, summary, evidence_id } => {
            let found = chat.iter_mut().rev().find_map(|i| match i {
                ChatItem::Tool { call_id: c, ok: o, summary: s, evidence_id: e, .. } if *c == call_id => Some((o, s, e)),
                _ => None,
            });
            match found {
                Some((o, s, e)) => {
                    *o = Some(ok);
                    *s = summary;
                    *e = evidence_id;
                }
                None => chat.push(ChatItem::Tool { call_id, tool, ok: Some(ok), summary, evidence_id, evidence: None }),
            }
        }
        AgentEvent::ConsentNeeded { call_id, tool, title, explanation } => {
            end_streaming(chat);
            chat.push(ChatItem::Consent { call_id, tool, title, explanation, answer: None });
        }
        AgentEvent::TurnDone { backend, .. } => {
            if matches!(chat.last(), Some(ChatItem::Status(_))) {
                chat.pop();
            }
            for item in chat.iter_mut().rev() {
                if let ChatItem::Assistant { streaming, backend: b, .. } = item {
                    *streaming = false;
                    if b.is_none() {
                        *b = Some(backend.clone());
                    }
                    break;
                }
                if matches!(item, ChatItem::User(_)) {
                    break;
                }
            }
        }
        AgentEvent::Error { message, remediation, log } => {
            end_streaming(chat);
            if matches!(chat.last(), Some(ChatItem::Status(_))) {
                chat.pop();
            }
            chat.push(ChatItem::Error { message, remediation, log });
        }
    }
}

fn end_streaming(chat: &mut [ChatItem]) {
    if let Some(ChatItem::Assistant { streaming, .. }) = chat.last_mut() {
        *streaming = false;
    }
}

/// Plain-words rendering of a non-evidence tool result (status values).
pub fn describe_value(v: &Value) -> String {
    match v {
        Value::Object(m) => m
            .iter()
            .take(24)
            .map(|(k, v)| {
                let val = match v {
                    Value::String(s) => s.clone(),
                    Value::Null => "none".into(),
                    Value::Array(a) => crate::i18n::trf("{n} item(s)", &[("n", &a.len().to_string())]),
                    Value::Object(_) => "…".into(),
                    other => other.to_string(),
                };
                format!("- **{}**: {}", k.replace('_', " "), val)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

/// Questions to suggest for a kind of target.
pub fn suggestions(kind: Option<TargetKind>) -> Vec<&'static str> {
    use TargetKind::*;
    match kind {
        None => vec!["What can Craft Anything do?", "How do I open an app?", "Is it safe to look inside programs?"],
        Some(Elf | Pe | MachO | MachOFat | DosCom | RawBinary) => {
            vec!["What does this program do?", "What does it send over the internet?", "Is it safe to run?", "Which files does it read or write?"]
        }
        Some(Managed | Jar | JavaClass) => vec!["What does this program do?", "What are its main parts?", "Does it talk to the internet?"],
        Some(Sqlite) => vec!["What does this app store?", "What tables are in here?", "Is anything sensitive saved in plain text?"],
        Some(Apk | Dex) => vec!["What does this app do?", "What permissions does it ask for?", "Which servers does it talk to?", "Does it track me?"],
        Some(Asar | Script) => vec!["How is this app built?", "Which websites does it load?", "How do the window and the page talk to each other?"],
        Some(Wasm) => vec!["What does this module do?", "What does it need from the page?"],
        Some(Directory) => vec!["What's in this folder?", "Which file is the main program?", "How is this app put together?"],
        Some(Zip) => vec!["What's inside this archive?", "Is anything in here a program?"],
        Some(NetworkCapture) => vec!["Which servers were contacted?", "What data was sent?"],
        Some(Plist) => vec!["What settings are in here?"],
        Some(Evm) => vec!["What does this contract do?", "Who can move funds?"],
        Some(Unknown) => vec!["What kind of file is this?", "Is there any readable text inside?"],
    }
}

/// Beginner Ask screen: target card on the left, conversation in the centre.
pub fn ask_screen(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let side_w = 300.0_f32.min(ui.available_width() * 0.3);
    // What's open sits on the right; the app's sidebar is on the left.
    egui::Panel::right("ask-side")
        .exact_size(side_w)
        .resizable(false)
        .frame(egui::Frame::new().fill(t.canvas).inner_margin(Margin { left: 8, right: 16, top: 16, bottom: 16 }))
        .show(ui, |ui| {
            target_card(app, ui);
        });
    egui::CentralPanel::default().frame(egui::Frame::new().fill(t.canvas).inner_margin(PANE_MARGIN)).show(ui, |ui| {
        conversation(app, ui, false);
    });
}

fn target_card(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    side_card(&t).show(ui, |ui| {
        ui.set_width(ui.available_width());
        match app.view.active_target().cloned() {
            Some(target) => {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(44.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 22.0, t.secondary_fill);
                    icons::paint(ui, rect, crate::kind_icon(target.kind), 22.0, t.text);
                    ui.vertical(|ui| {
                        ui.add(egui::Label::new(theme::heading(&target.name, theme::CARD)).truncate());
                        ui.label(RichText::new(crate::human_bytes(target.bytes.len())).color(t.text_faint).size(theme::CAPTION));
                    });
                });
                ui.add_space(10.0);
                widgets::caption(ui, "What is it");
                ui.label(RichText::new(tr(target.kind.plain())).size(theme::BODY));
                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    if app.view.clean_room {
                        widgets::pill(ui, "Clean room", t.success, t.raised).on_hover_text(tr("Nothing inside it is read."));
                    } else {
                        widgets::pill(ui, "Deep", t.accent_ink, t.accent_soft);
                    }
                    widgets::pill(ui, &format!("{} {}", app.view.ledger_len, tr("findings")), t.text_dim, t.raised);
                });
            }
            None => {
                widgets::empty_state(ui, "folder-open", "Nothing open yet", "Drop a file or app anywhere in this window.");
                ui.vertical_centered(|ui| {
                    if widgets::primary_icon(ui, Some("folder-open"), "Open an app or file", true).clicked() {
                        app.pick(crate::Pick::OpenFile);
                    }
                });
            }
        }
    });
    ui.add_space(10.0);
    side_card(&t).show(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::caption(ui, "Try asking");
        ui.add_space(4.0);
        let kind = app.view.active_target().map(|t| t.kind);
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            for q in suggestions(kind) {
                if widgets::suggestion(ui, q).clicked() {
                    app.send(&tr(q));
                }
            }
        });
    });
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        widgets::toggle(ui, &mut app.state.explain_new, "Explain like I'm new")
            .on_hover_text(tr("Plain words, short sentences, and every technical word explained."));
    });
    ui.add_space(10.0);
    if widgets::secondary(ui, Some("download"), "Export a report").on_hover_text(tr("Everything found so far, as a Markdown file.")).clicked() {
        app.export_report();
    }
}

/// A details panel's card: a rounded fill a step above the page, no edge.
pub(crate) fn side_card(t: &Tokens) -> egui::Frame {
    egui::Frame::new().fill(t.card).corner_radius(t.r_lg()).inner_margin(Margin::same(16))
}

/// The chat panes' margins: the floating prompt box sits 14 above the bottom edge.
pub const PANE_MARGIN: Margin = Margin { left: 14, right: 14, top: 10, bottom: 14 };
/// Between the last message and the prompt box.
pub const COMPOSER_GAP: f32 = 8.0;
/// Between messages (on top of the item spacing).
pub const MESSAGE_GAP: f32 = 6.0;
/// How far the history runs under a fade above the prompt box.
pub const BOTTOM_FADE: f32 = 20.0;

/// The conversation with its composer. `dense` = Experienced agent console.
pub fn conversation(app: &mut CraftApp, ui: &mut Ui, dense: bool) {
    let width = ui.available_width();
    let max_w = if dense { width } else { width.min(COLUMN_W) };
    let pad = ((width - max_w) / 2.0).max(0.0);
    // The history gets exactly what the composer leaves (its height is measured last frame), so
    // a panel never grows to fit us.
    let comp_id = ui.id().with(("composer-h", dense));
    let comp_h = ui.data(|d| d.get_temp::<f32>(comp_id)).unwrap_or(if dense { 48.0 } else { 70.0 });
    // The item spacing after the history counts too. Anything here that comes out taller than the
    // space given makes the agent console's panel grow by that much every frame, until it reaches
    // its maximum and the top of its tab strip is cut off.
    let rest = Vec2::new(width, (ui.available_height() - comp_h - COMPOSER_GAP - ui.spacing().item_spacing.y).max(40.0));
    let area = ui.allocate_ui_with_layout(rest, egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_height(rest.y);
        history(app, ui, dense, pad, max_w)
    });
    // The history melts into the page just above the box.
    if !dense {
        let r = area.response.rect;
        let t = Tokens::get(ui.ctx());
        fade(ui.painter(), egui::Rect::from_x_y_ranges(r.x_range(), (r.bottom() - BOTTOM_FADE)..=r.bottom()), t.canvas, false, 0.0);
    }
    ui.add_space(COMPOSER_GAP);
    // Top-aligned: `horizontal` centres in its row, which put the box 12 points below the gap.
    let r = ui.horizontal_top(|ui| {
        ui.add_space(pad);
        // Stacked, so the row measures the box and the line under it.
        ui.vertical(|ui| composer(app, ui, max_w, dense));
    });
    remember_composer_height(ui, comp_id, comp_h, r.response.rect.height());
}

/// Keep the prompt box's height for the next frame's layout. When it changed (a line added or
/// removed, a note appeared), this frame was laid out with the old height, so it's thrown away
/// and laid out again before it's shown: otherwise the box and the feed jump while typing.
pub fn remember_composer_height(ui: &mut Ui, id: egui::Id, used: f32, measured: f32) {
    ui.data_mut(|d| d.insert_temp(id, measured));
    if (measured - used).abs() > 0.5 {
        ui.ctx().request_discard("the prompt box changed height");
    }
}

fn history(app: &mut CraftApp, ui: &mut Ui, dense: bool, pad: f32, max_w: f32) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(pad);
            ui.vertical(|ui| {
                ui.set_width(max_w);
                if app.state.chat.is_empty() {
                    greeting(app, ui, dense);
                }
                let mut actions = Vec::new();
                let beginner = app.beginner();
                let last_proof = app.state.chat.iter().rposition(|i| matches!(i, ChatItem::Proof { .. }));
                // The gaps between messages are set here (closer within a speaker's run); what's
                // inside each one keeps the usual spacing.
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing.y = 0.0;
                if !dense {
                    ui.add_space(8.0);
                }
                let mut last = None;
                for (i, item) in app.state.chat.iter().enumerate() {
                    let open = !dense && Some(i) == last_proof;
                    let side = speaker(item);
                    if i > 0 {
                        ui.add_space(if dense {
                            spacing.y
                        } else if last == Some(side) && side != Speaker::Card {
                            SAME_SPEAKER_GAP
                        } else {
                            NEW_SPEAKER_GAP
                        });
                    }
                    last = Some(side);
                    ui.push_id(i, |ui| {
                        ui.spacing_mut().item_spacing = spacing;
                        item_ui(ui, item, beginner, dense, open, &mut actions)
                    });
                }
                ui.add_space(if dense { 4.0 } else { NEW_SPEAKER_GAP });
                ui.spacing_mut().item_spacing = spacing;
                if !dense {
                    ui.add_space(BOTTOM_FADE / 2.0);
                }
                // Only for this conversation: a build or a download elsewhere isn't an answer.
                if app.answering() && !matches!(app.state.chat.last(), Some(ChatItem::Assistant { streaming: true, .. })) {
                    if dense {
                        widgets::thinking(ui);
                    } else {
                        widgets::working(ui, "Working on your answer", None);
                    }
                }
                for a in actions {
                    match a {
                        ChatAction::Consent(id, ans) => app.answer_consent(&id, ans),
                        ChatAction::OpenLog(path) => app.open_log(&path),
                        ChatAction::AddBrain => {
                            app.state.settings_open = true;
                            app.state.settings_tab = crate::state::SettingsTab::Ai;
                            app.detect_backends();
                        }
                    }
                }
            });
        });
    });
}

fn greeting(app: &CraftApp, ui: &mut Ui, dense: bool) {
    let t = Tokens::get(ui.ctx());
    if dense {
        ui.label(RichText::new(tr("Ask the agent about this target. It cites evidence for every claim.")).color(t.text_faint));
        return;
    }
    ui.add_space(24.0);
    let name = app.view.active_target().map(|t| t.name.clone());
    let title = match &name {
        Some(n) => crate::i18n::trf("What would you like to know about {name}?", &[("name", n)]),
        None => tr("What would you like to find out?"),
    };
    ui.label(theme::heading(title, theme::TITLE));
    ui.add_space(4.0);
    let sub = if app.state.backend.is_some() {
        "Ask in your own words. Every answer shows how sure we are, and the proof."
    } else {
        "No AI brain is set up, so answers come straight from the built-in engines. Add one any time for plain-words explanations."
    };
    ui.label(RichText::new(tr(sub)).color(t.text_dim));
    ui.add_space(16.0);
}

fn composer(app: &mut CraftApp, ui: &mut Ui, width: f32, dense: bool) {
    let answering = app.answering();
    let hint = if dense { tr("Ask the agent…") } else { tr("Ask anything, e.g. \"What does this program do?\"") };
    ui.allocate_ui(Vec2::new(width, 0.0), |ui| {
        let history = || app.state.chat.iter().filter_map(|i| if let ChatItem::User(text) = i { Some(text.clone()) } else { None }).collect();
        let out = crate::prompt_box::show(
            ui,
            &mut app.state.input,
            &[],
            crate::prompt_box::PromptBox {
                name: "Ask a question",
                hint,
                rows: (1, if dense { 4 } else { 8 }),
                attach: None,
                stop_button: answering.then_some("Stop"),
                send: !answering,
                recall: false,
                history: Some(&history),
                font_size: if dense { 14.0 } else { 16.0 },
            },
        );
        if out.stop {
            app.stop();
        }
        // One answer at a time; what's typed meanwhile stays in the box.
        if out.send && !answering {
            let text = std::mem::take(&mut app.state.input);
            app.send(&text);
        }
    });
}

/// Who an item comes from, for the gaps between them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Speaker {
    Person,
    App,
    /// Questions, errors and offers: always set apart.
    Card,
}

fn speaker(item: &ChatItem) -> Speaker {
    match item {
        ChatItem::User(_) => Speaker::Person,
        ChatItem::Assistant { .. } | ChatItem::Status(_) | ChatItem::Tool { .. } | ChatItem::Proof { .. } => Speaker::App,
        ChatItem::Consent { .. } | ChatItem::Error { .. } | ChatItem::AddBrain => Speaker::Card,
    }
}

enum ChatAction {
    Consent(String, ConsentAnswer),
    AddBrain,
    OpenLog(std::path::PathBuf),
}

fn item_ui(ui: &mut Ui, item: &ChatItem, beginner: bool, dense: bool, open: bool, actions: &mut Vec<ChatAction>) {
    let t = Tokens::get(ui.ctx());
    match item {
        ChatItem::User(text) => user_bubble(ui, text, dense),
        ChatItem::Assistant { text, backend, streaming } => {
            agent_bubble(ui, dense, |ui| {
                rich_text(ui, text, beginner && !dense);
                if *streaming {
                    widgets::thinking(ui);
                }
                if let Some(b) = backend {
                    ui.add_space(4.0);
                    ui.label(RichText::new(crate::i18n::trf("Answered by {name}", &[("name", b)])).color(t.text_faint).size(theme::CAPTION));
                }
            });
        }
        ChatItem::Status(text) => {
            activity_row(ui, "activity", RichText::new(text).color(t.text_dim).size(theme::SMALL));
        }
        ChatItem::Tool { tool, ok, summary, evidence, .. } => {
            ui.horizontal_wrapped(|ui| {
                let (icon, col) = match ok {
                    None => ("loader", t.text_faint),
                    Some(true) => ("check", t.success),
                    Some(false) => ("triangle-alert", t.danger),
                };
                egui::Frame::new().fill(t.bubble_agent).corner_radius(t.r_pill()).inner_margin(Margin::symmetric(10, 3)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        icons::show(ui, icon, 13.0, col);
                        ui.label(RichText::new(tool.replace('_', " ")).font(theme::mono(12.5)).color(t.text_dim));
                    });
                });
                if !summary.is_empty() {
                    ui.label(RichText::new(summary).color(t.text_dim).size(theme::CAPTION));
                }
            });
            if let Some(e) = evidence {
                ui.add_space(4.0);
                evidence_ui::proof_card(ui, e, dense, false);
            }
        }
        ChatItem::Proof { intro, evidence } => {
            if !intro.is_empty() {
                ui.label(RichText::new(intro).color(t.text_dim));
                ui.add_space(4.0);
            }
            evidence_ui::proof_card(ui, evidence, dense, open);
        }
        ChatItem::Consent { call_id, title, explanation, answer, .. } => {
            notice(&t, None).show(ui, |ui| {
                ui.set_width(ui.available_width());
                notice_title(ui, "shield", t.warning, crate::i18n::trf("May I run \"{title}\"?", &[("title", title)]));
                ui.label(RichText::new(explanation).color(t.text_dim));
                ui.label(
                    RichText::new(tr("This runs code from the target or changes a device. Nothing runs until you say so."))
                        .color(t.text_faint)
                        .size(theme::CAPTION),
                );
                ui.add_space(6.0);
                match answer {
                    None => {
                        ui.horizontal(|ui| {
                            if widgets::primary(ui, "Allow once").clicked() {
                                actions.push(ChatAction::Consent(call_id.clone(), ConsentAnswer::Once));
                            }
                            if widgets::secondary(ui, None, "Always for this project").clicked() {
                                actions.push(ChatAction::Consent(call_id.clone(), ConsentAnswer::AlwaysForProject));
                            }
                            if widgets::secondary(ui, None, "Don't allow").clicked() {
                                actions.push(ChatAction::Consent(call_id.clone(), ConsentAnswer::Deny));
                            }
                        });
                    }
                    Some(a) => {
                        let text = match a {
                            ConsentAnswer::Once => "Allowed once.",
                            ConsentAnswer::AlwaysForProject => "Allowed for this project.",
                            ConsentAnswer::Deny => "Not allowed.",
                        };
                        ui.label(RichText::new(tr(text)).color(t.text_faint));
                    }
                }
            });
        }
        ChatItem::Error { message, remediation, log } => {
            notice(&t, Some(t.danger)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                notice_title(ui, "triangle-alert", t.danger, message.clone());
                if let Some(r) = remediation {
                    ui.add(egui::Label::new(RichText::new(r).color(t.text_dim)).wrap());
                }
                if let Some(l) = log
                    && widgets::open_log_button(ui, l)
                {
                    actions.push(ChatAction::OpenLog(l.clone()));
                }
            });
        }
        ChatItem::AddBrain => {
            notice(&t, None).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    icons::show(ui, "sparkles", 18.0, t.text_dim);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(tr("Want answers in plain words?")).font(theme::semibold(theme::BODY)));
                        ui.label(
                            RichText::new(tr("Add an AI brain and I'll read this evidence for you, answer follow-up questions and point at the proof."))
                                .color(t.text_dim),
                        );
                        ui.add_space(4.0);
                        if widgets::primary_icon(ui, Some("plus"), "Add a brain", true).clicked() {
                            actions.push(ChatAction::AddBrain);
                        }
                    });
                });
            });
        }
    }
}

// ---- the chat look ------------------------------------------------------------------------------
// Shared by the Ask screen, the agent console and the Project screen's feed.

/// The conversation column's widest, centred in its pane.
pub const COLUMN_W: f32 = 820.0;
/// Between two messages from the same side, and where the speaker changes or a card comes.
pub const SAME_SPEAKER_GAP: f32 = 4.0;
pub const NEW_SPEAKER_GAP: f32 = 12.0;

/// A message bubble: the person's in the stronger grey, the app's and the builder's a step
/// quieter. Rounded, no edge, no tail.
pub(crate) fn bubble(t: &Tokens, user: bool, dense: bool) -> egui::Frame {
    let fill = if user { t.bubble_user } else { t.bubble_agent };
    let m = if dense { Margin::symmetric(10, 6) } else { Margin::symmetric(12, 8) };
    egui::Frame::new().fill(fill).corner_radius(t.r_lg()).inner_margin(m)
}

/// How wide a bubble may grow: most of the column, leaving the other side clear.
fn bubble_max(ui: &Ui, dense: bool) -> f32 {
    let w = ui.available_width();
    if dense { w } else { (w * 0.8).max(120.0).min(w) }
}

/// The person's message on the right. Labels in a right-to-left row don't wrap by themselves,
/// so a long prompt would run off the side: wrap it inside the bubble.
pub(crate) fn user_bubble(ui: &mut Ui, text: &str, dense: bool) {
    let t = Tokens::get(ui.ctx());
    let max = bubble_max(ui, dense).min(USER_BUBBLE_TEXT_W);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        // Laid out first, so the bubble is exactly as wide as its longest line.
        let galley =
            egui::WidgetText::from(RichText::new(text).color(t.text)).into_galley(ui, Some(egui::TextWrapMode::Wrap), max, egui::FontSelection::Default);
        bubble(&t, true, dense).show(ui, |ui| {
            ui.set_max_width(galley.size().x.ceil());
            // The bubble sits on the right; its lines still read from the left.
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.add(egui::Label::new(galley));
            });
        });
    });
}

/// The widest a line of the person's message runs.
const USER_BUBBLE_TEXT_W: f32 = 560.0;

/// The app's (or the builder's) message on the left, as wide as its text up to most of the column.
pub(crate) fn agent_bubble<R>(ui: &mut Ui, dense: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let t = Tokens::get(ui.ctx());
    let max = bubble_max(ui, dense);
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
        bubble(&t, false, dense)
            .show(ui, |ui| {
                ui.set_max_width(max - 24.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), add).inner
            })
            .inner
    })
    .inner
}

/// A quiet line about what's happening ("Looking inside it…"): a small faint glyph and dim
/// words, no box. It wraps instead of running off the side.
pub(crate) fn activity_row(ui: &mut Ui, icon: &str, text: RichText) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add_space(6.0);
        let line = ui.text_style_height(&egui::TextStyle::Body).max(14.0);
        let (r, _) = ui.allocate_exact_size(Vec2::new(12.0, line), egui::Sense::hover());
        icons::paint(ui, r, icon, 12.0, t.text_faint);
        ui.add(egui::Label::new(text).wrap());
    });
}

/// A centred, faint line between messages ("Started 12 minutes ago"). No rules.
pub(crate) fn timestamp(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.add_space(10.0);
    ui.vertical_centered(|ui| ui.label(RichText::new(text).color(t.text_faint).size(theme::CAPTION)));
    ui.add_space(4.0);
}

/// A card in the conversation (a question, a check, a result): a rounded fill a step above the
/// page, no edge. `tone` tints it (danger for a failure).
pub(crate) fn notice(t: &Tokens, tone: Option<Color32>) -> egui::Frame {
    let base = if t.dark() { t.raised } else { t.card };
    let fill = tone.map_or(base, |c| theme::mix(base, c, 0.12));
    egui::Frame::new().fill(fill).corner_radius(t.r_lg()).inner_margin(Margin::same(14))
}

/// A notice's first line: a small tinted glyph and the title.
pub(crate) fn notice_title(ui: &mut Ui, icon: &str, tint: Color32, title: impl Into<String>) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let h = ui.text_style_height(&egui::TextStyle::Body).max(16.0);
        let (r, _) = ui.allocate_exact_size(Vec2::new(16.0, h), egui::Sense::hover());
        icons::paint(ui, r, icon, 15.0, tint);
        ui.add(egui::Label::new(RichText::new(title).font(theme::semibold(theme::BODY))).wrap());
    });
}

/// A two-or-more way switch: a pill track with the chosen option as a lighter pill on it.
/// Returns the newly picked index when it changes. Options are named for screen readers.
pub(crate) fn pill_switch(ui: &mut Ui, options: &[&str], selected: usize) -> Option<usize> {
    let t = Tokens::get(ui.ctx());
    let font = theme::medium(theme::CAPTION);
    let galleys: Vec<_> = options.iter().map(|o| ui.painter().layout_no_wrap(tr(o), font.clone(), t.text)).collect();
    let (pad, gap, inner_h) = (3.0, 2.0, 24.0);
    let widths: Vec<f32> = galleys.iter().map(|g| g.size().x + 24.0).collect();
    let total = widths.iter().sum::<f32>() + gap * widths.len().saturating_sub(1) as f32 + pad * 2.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total, inner_h + pad * 2.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, t.r_pill(), t.secondary_fill);
    let mut x = rect.left() + pad;
    let mut picked = None;
    for (i, (g, w)) in galleys.into_iter().zip(widths).enumerate() {
        let seg = egui::Rect::from_min_size(egui::pos2(x, rect.top() + pad), Vec2::new(w, inner_h));
        let label = options.get(i).copied().unwrap_or_default();
        let resp = ui.interact(seg, ui.id().with(("pill-switch", label)), egui::Sense::click());
        let on = i == selected;
        resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, tr(label)));
        if on {
            if !t.dark() {
                ui.painter().add(egui::Shadow { offset: [0, 1], blur: 3, spread: 0, color: Color32::from_black_alpha(30) }.as_shape(seg, t.r_pill()));
            }
            ui.painter().rect_filled(seg, t.r_pill(), thumb(&t));
        } else {
            let h = widgets::hover_t(ui, &resp);
            ui.painter().rect_filled(seg, t.r_pill(), theme::mix(Color32::TRANSPARENT, faint_overlay(&t, 0.06), h));
        }
        let color = if on { t.text } else { t.text_dim };
        ui.painter().galley_with_override_text_color(seg.center() - g.size() / 2.0, g, color);
        if resp.has_focus() {
            ui.painter().rect_stroke(seg, t.r_pill(), Stroke::new(2.0, t.text_dim), egui::StrokeKind::Inside);
        }
        let resp = if on { resp } else { resp.on_hover_cursor(egui::CursorIcon::PointingHand) };
        if resp.clicked() && !on {
            picked = Some(i);
        }
        x += w + gap;
    }
    picked
}

/// The chosen option's pill: a lighter grey on dark, white on light.
fn thumb(t: &Tokens) -> Color32 {
    if t.dark() { theme::mix(t.secondary_fill, t.text, 0.15) } else { Color32::WHITE }
}

/// The text colour at `alpha` (0..1): hover tints and outlines.
pub(crate) fn faint_overlay(t: &Tokens, alpha: f32) -> Color32 {
    t.text.gamma_multiply(alpha)
}

/// What a round button looks like.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Round {
    /// White (black in light) with a dark glyph: send.
    Primary,
    /// The primary one with nothing to do yet: dimmed.
    PrimaryDim,
    /// A grey disc (attach, stop) from the composer's palette.
    Control,
    /// Page-coloured with a hairline ring (the details toggle).
    Ringed,
}

/// The glyphs round buttons draw (painted, so they're crisp at any size).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Glyph {
    Plus,
    /// An x: take back, remove.
    Cross,
    ArrowUp,
    Stop,
    Pause,
    /// « (`true`) or ».
    Chevrons(bool),
}

/// A round button of `size` with a painted glyph, named `name` for screen readers and hover.
pub(crate) fn round_button(ui: &mut Ui, size: f32, look: Round, glyph: Glyph, name: &str) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let sense = if look == Round::PrimaryDim { egui::Sense::hover() } else { egui::Sense::click() };
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), sense);
    let h = widgets::hover_t(ui, &resp);
    let down = resp.is_pointer_button_down_on();
    let (fill, ink) = match look {
        Round::Primary => (theme::mix(t.accent, t.text_dim, if down { 0.25 } else { 0.12 * h }), t.on_accent),
        Round::PrimaryDim => (t.secondary_fill, t.text_faint),
        Round::Control => (theme::mix(t.composer_control, t.text, if down { 0.12 } else { 0.06 * h }), t.text),
        Round::Ringed => (theme::mix(t.canvas, t.text, if down { 0.1 } else { 0.05 * h }), t.text),
    };
    let c = rect.center();
    let r = size / 2.0;
    ui.painter().circle_filled(c, r, fill);
    if look == Round::Ringed {
        ui.painter().circle_stroke(c, r - 0.5, Stroke::new(1.0, t.chrome_stroke));
    }
    paint_glyph(ui.painter(), rect, glyph, ink);
    if resp.has_focus() {
        ui.painter().circle_stroke(c, r + 2.0, Stroke::new(2.0, t.text_dim));
    }
    let label = tr(name);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, look != Round::PrimaryDim, &label));
    let resp = if look == Round::PrimaryDim { resp } else { resp.on_hover_cursor(egui::CursorIcon::PointingHand) };
    resp.on_hover_text(label)
}

fn paint_glyph(p: &egui::Painter, rect: egui::Rect, glyph: Glyph, ink: Color32) {
    let c = rect.center();
    let s = rect.width();
    let stroke = Stroke::new((s / 14.0).clamp(1.4, 2.2), ink);
    let line = |a: egui::Pos2, b: egui::Pos2| p.line_segment([a, b], stroke);
    match glyph {
        Glyph::Plus => {
            let a = s * 0.24;
            line(c - Vec2::new(a, 0.0), c + Vec2::new(a, 0.0));
            line(c - Vec2::new(0.0, a), c + Vec2::new(0.0, a));
        }
        Glyph::Cross => {
            let a = s * 0.17;
            line(c - Vec2::splat(a), c + Vec2::splat(a));
            line(c + Vec2::new(-a, a), c + Vec2::new(a, -a));
        }
        Glyph::ArrowUp => {
            let a = s * 0.25;
            line(c + Vec2::new(0.0, a), c - Vec2::new(0.0, a));
            line(c - Vec2::new(0.0, a), c + Vec2::new(-a * 0.85, -a * 0.15));
            line(c - Vec2::new(0.0, a), c + Vec2::new(a * 0.85, -a * 0.15));
        }
        Glyph::Stop => {
            p.rect_filled(egui::Rect::from_center_size(c, Vec2::splat(s * 0.32)), 2.0, ink);
        }
        Glyph::Pause => {
            let (w, h, gap) = (s * 0.11, s * 0.36, s * 0.08);
            for dx in [-(gap + w / 2.0), gap + w / 2.0] {
                p.rect_filled(egui::Rect::from_center_size(c + Vec2::new(dx, 0.0), Vec2::new(w, h)), 1.0, ink);
            }
        }
        Glyph::Chevrons(left) => {
            let (a, d) = (s * 0.14, s * 0.15);
            let dir = if left { -1.0 } else { 1.0 };
            for off in [-d / 2.0 - a * 0.2, d / 2.0 + a * 0.2] {
                let tip = c + Vec2::new(off + dir * a / 2.0, 0.0);
                let back = tip - Vec2::new(dir * a, 0.0);
                line(tip, back + Vec2::new(0.0, -a));
                line(tip, back + Vec2::new(0.0, a));
            }
        }
    }
}

/// A vertical fade over `rect`: `colour` at the top edge (`down`) or the bottom one, clear at the
/// other. Content scrolling under the floating header and composer melts into the page.
/// `hold`: how much of it, from the coloured edge, is solid.
pub(crate) fn fade(p: &egui::Painter, rect: egui::Rect, colour: Color32, down: bool, hold: f32) {
    let mut mesh = egui::Mesh::default();
    // Solid for the first `hold` of it (from the coloured edge), then three bands so it eases
    // rather than ramps.
    let hold = hold.clamp(0.0, 0.9);
    let stops = [(0.0, 1.0), (hold, 1.0), (hold + (1.0 - hold) * 0.45, 0.6), (1.0, 0.0)];
    for (i, (at, k)) in stops.iter().enumerate() {
        let y = egui::lerp(rect.top()..=rect.bottom(), if down { *at } else { 1.0 - at });
        let c = colour.gamma_multiply(*k);
        mesh.colored_vertex(egui::pos2(rect.left(), y), c);
        mesh.colored_vertex(egui::pos2(rect.right(), y), c);
        if i > 0 {
            let b = (i as u32) * 2;
            mesh.add_triangle(b - 2, b - 1, b);
            mesh.add_triangle(b - 1, b, b + 1);
        }
    }
    p.add(egui::Shape::mesh(mesh));
}

/// Light Markdown (headings, bullets, code fences, **bold**) with glossary hovers when `glossary`.
pub fn rich_text(ui: &mut Ui, text: &str, with_glossary: bool) {
    let t = Tokens::get(ui.ctx());
    let body = ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_else(|| egui::FontId::proportional(theme::SMALL));
    let mut in_code = false;
    let mut code = String::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_code {
                code_block(ui, &code, &t);
                code.clear();
            }
            in_code = !in_code;
            continue;
        }
        if in_code {
            code.push_str(line);
            code.push('\n');
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            ui.add_space(4.0);
            continue;
        }
        if let Some(h) = trimmed.strip_prefix("### ").or_else(|| trimmed.strip_prefix("## ")).or_else(|| trimmed.strip_prefix("# ")) {
            ui.label(RichText::new(h).font(theme::semibold(body.size + 2.0)));
            continue;
        }
        let (bullet, rest) = match trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            Some(r) => (true, r),
            None => (false, trimmed),
        };
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if bullet {
                ui.label(RichText::new("  •  ").color(t.text_dim));
            }
            inline(ui, rest, with_glossary, &body, &t);
        });
    }
    if in_code && !code.is_empty() {
        code_block(ui, &code, &t);
    }
}

fn inline(ui: &mut Ui, text: &str, with_glossary: bool, body: &egui::FontId, t: &Tokens) {
    // Split on ** and ` markers; alternate plain/bold and plain/code.
    let mut bold = false;
    for (bi, part) in text.split("**").enumerate() {
        if bi > 0 {
            bold = !bold;
        }
        for (ci, piece) in part.split('`').enumerate() {
            let code = ci % 2 == 1;
            if piece.is_empty() {
                continue;
            }
            if code {
                ui.label(RichText::new(piece).font(theme::mono(body.size - 1.0)).color(t.accent_ink).background_color(t.code_bg));
                continue;
            }
            let font = if bold { theme::semibold(body.size) } else { body.clone() };
            if with_glossary {
                for (run, meaning) in glossary::segments(piece) {
                    match meaning {
                        Some(m) => {
                            widgets::glossary_word(ui, &run, m, font.clone());
                        }
                        None => words(ui, &run, &font, t),
                    }
                }
            } else {
                words(ui, piece, &font, t);
            }
        }
    }
}

/// A run of text as one label: in a wrapping row egui starts it where the last one ended and
/// breaks it between words, so a paragraph isn't one widget per word.
fn words(ui: &mut Ui, text: &str, font: &egui::FontId, t: &Tokens) {
    if !text.is_empty() {
        ui.add(egui::Label::new(RichText::new(text).font(font.clone()).color(t.text)).wrap());
    }
}

fn code_block(ui: &mut Ui, code: &str, t: &Tokens) {
    egui::Frame::new().fill(t.code_bg).corner_radius(t.r()).inner_margin(Margin::same(10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(code.trim_end()).font(theme::mono(12.5)).color(t.text));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_fold_into_a_tidy_conversation() {
        let mut chat = vec![ChatItem::User("hi".into())];
        apply_event(&mut chat, AgentEvent::Status { text: "Looking…".into() });
        apply_event(&mut chat, AgentEvent::Status { text: "Still looking…".into() });
        assert_eq!(chat.len(), 2);
        apply_event(&mut chat, AgentEvent::TextDelta { text: "Hel".into() });
        apply_event(&mut chat, AgentEvent::TextDelta { text: "lo".into() });
        assert_eq!(chat.len(), 2, "status replaced by the answer");
        apply_event(&mut chat, AgentEvent::ToolStarted { call_id: "c1".into(), tool: "core_target_triage".into(), input: Value::Null });
        apply_event(
            &mut chat,
            AgentEvent::ToolFinished {
                call_id: "c1".into(),
                tool: "core_target_triage".into(),
                ok: true,
                summary: "done".into(),
                evidence_id: Some("ev".into()),
            },
        );
        apply_event(&mut chat, AgentEvent::ConsentNeeded { call_id: "c2".into(), tool: "run".into(), title: "Run it".into(), explanation: "runs".into() });
        apply_event(&mut chat, AgentEvent::TurnDone { backend: "Claude".into(), input_tokens: 1, output_tokens: 2, cost_usd: None, cost_is_estimate: false });
        match &chat[1] {
            ChatItem::Assistant { text, streaming, backend } => {
                assert_eq!(text, "Hello");
                assert!(!streaming);
                assert_eq!(backend.as_deref(), Some("Claude"));
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(&chat[2], ChatItem::Tool { ok: Some(true), evidence_id: Some(_), .. }));
        assert!(matches!(&chat[3], ChatItem::Consent { answer: None, .. }));
    }

    #[test]
    fn every_kind_has_suggestions() {
        for k in [TargetKind::Elf, TargetKind::Apk, TargetKind::Asar, TargetKind::Unknown, TargetKind::Directory] {
            assert!(suggestions(Some(k)).len() >= 2);
        }
        assert!(!suggestions(None).is_empty());
        assert!(describe_value(&serde_json::json!({"a_b": 1})).contains("a b"));
    }
}
