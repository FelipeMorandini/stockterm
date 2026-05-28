//! In-app dashboard layout editor (Issue #208 / §71).

use crate::app::app_error::{AppError, ErrorSourceDomain};
use crate::app::keyboard::letter_key_plain;
use crate::app::layout::centered_rect;
use crate::app::styles::ResolvedTheme;
use crate::app::App;
use crate::app::Tab;
use crate::config::keymap::{Action, BindingLayer};
use crate::models::dashboard::{
    allocate_dashboard_name, allocate_dashboard_pane_id, clone_preset_with_name,
    format_dashboard_validation_errors, preset_dual_watchlist, preset_market_overview,
    validate_dashboard_definition, validate_dashboard_name_unique, validate_dashboard_pane,
    DashboardDefinition, DashboardPane, DashboardPaneKind, DashboardPaneOptions,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

/// Wizard screen for the dashboard editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardEditorScreen {
    PickDashboard,
    NewFromPreset,
    EditLayout,
    EditPane,
    ConfirmDiscard,
}

/// Focus target on the main edit screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardEditLayoutFocus {
    Rows,
    Cols,
    PaneList,
    SetActive,
    Save,
}

/// Focus target on the pane sub-form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardPaneFormField {
    Kind,
    Row,
    Col,
    RowSpan,
    ColSpan,
    Title,
    MaxRows,
}

/// Mutable pane fields on [`DashboardEditorScreen::EditPane`].
#[derive(Debug, Clone)]
pub struct DashboardPaneForm {
    pub pane_index: usize,
    pub kind: DashboardPaneKind,
    pub row_buf: String,
    pub col_buf: String,
    pub row_span_buf: String,
    pub col_span_buf: String,
    pub title_buf: String,
    pub max_rows_buf: String,
    pub focused: DashboardPaneFormField,
    /// Unsaved edits on the pane sub-form (Issue #208).
    pub dirty: bool,
}

/// In-app dashboard editor state (Issue #208 / §71).
#[derive(Debug, Clone)]
pub struct DashboardEditor {
    pub screen: DashboardEditorScreen,
    pub draft: DashboardDefinition,
    pub editing_index: Option<usize>,
    pub dirty: bool,
    pub selected_pane: usize,
    pub remove_armed: bool,
    pub inline_error: Option<String>,
    pub pane_form: Option<DashboardPaneForm>,
    pub set_as_active: bool,
    pub layout_focus: DashboardEditLayoutFocus,
    /// Pick list: dashboards + synthetic "new" row.
    pub pick_index: usize,
    pub preset_index: usize,
    /// Screen to restore when user declines [`DashboardEditorScreen::ConfirmDiscard`].
    pub confirm_discard_return: Option<DashboardEditorScreen>,
    /// Stable allocated name while the pick-list **New…** row is highlighted (preview only).
    pub new_dashboard_preview_name: Option<String>,
    /// Pre-built modal text; refreshed on editor state changes (§71 / `rust_tui.mdc`).
    pub overlay_lines: Vec<String>,
}

const PRESET_LABELS: [&str; 2] = ["dual_watchlist", "market_overview"];

fn preset_by_index(index: usize) -> DashboardDefinition {
    match index {
        0 => preset_dual_watchlist(),
        _ => preset_market_overview(),
    }
}

/// Preview draft for the highlighted pick-list row (Issue #208).
fn pick_preview_draft(
    config: &crate::config::Config,
    pick_index: usize,
    new_dashboard_preview_name: Option<&str>,
) -> DashboardDefinition {
    let count = config.dashboards.len();
    if pick_index < count {
        config.dashboards[pick_index].clone()
    } else {
        let name = new_dashboard_preview_name
            .map(str::to_string)
            .unwrap_or_else(|| allocate_dashboard_name(&config.dashboards));
        clone_preset_with_name(&preset_dual_watchlist(), &name)
    }
}

/// Preview draft for the highlighted preset row (Issue #208).
fn preset_preview_draft(
    dashboards: &[DashboardDefinition],
    preset_index: usize,
) -> DashboardDefinition {
    let preset_idx = preset_index.min(PRESET_LABELS.len().saturating_sub(1));
    let name = allocate_dashboard_name(dashboards);
    clone_preset_with_name(&preset_by_index(preset_idx), &name)
}

fn sync_pick_preview(editor: &mut DashboardEditor, config: &crate::config::Config) {
    let count = config.dashboards.len();
    if editor.pick_index >= count {
        if editor.new_dashboard_preview_name.is_none() {
            editor.new_dashboard_preview_name = Some(allocate_dashboard_name(&config.dashboards));
        }
    } else {
        editor.new_dashboard_preview_name = None;
    }
    editor.draft = pick_preview_draft(
        config,
        editor.pick_index,
        editor.new_dashboard_preview_name.as_deref(),
    );
    editor.refresh_validation_hint();
}

fn sync_preset_preview(editor: &mut DashboardEditor, dashboards: &[DashboardDefinition]) {
    editor.draft = preset_preview_draft(dashboards, editor.preset_index);
    editor.refresh_validation_hint();
}

impl DashboardEditor {
    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.inline_error = None;
    }

    fn refresh_validation_hint(&mut self) {
        self.inline_error = match validate_dashboard_definition(&self.draft) {
            Ok(()) => None,
            Err(errs) => Some(format_dashboard_validation_errors(&errs)),
        };
    }
}

/// Draw centered editor overlay (Issue #208 / §71).
pub fn draw_dashboard_editor_overlay(f: &mut Frame, app: &App, area: Rect, rt: ResolvedTheme) {
    let Some(editor) = &app.dashboard_editor else {
        return;
    };
    let popup = centered_rect(area, 85, 70);
    f.render_widget(Clear, popup);
    let block = Block::default()
        .title("Dashboard editor")
        .borders(Borders::ALL)
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let text: Vec<Line> = editor
        .overlay_lines
        .iter()
        .map(|s| Line::from(Span::styled(s.as_str(), rt.fg_border())))
        .collect();
    f.render_widget(Paragraph::new(text), inner);
}

fn editor_lines(app: &App, editor: &DashboardEditor) -> Vec<String> {
    let mut out = Vec::new();
    match editor.screen {
        DashboardEditorScreen::PickDashboard => {
            out.push("Select dashboard".into());
            out.push("j/k move · Enter open · Esc close".into());
            let count = app.config.dashboards.len();
            for (i, def) in app.config.dashboards.iter().enumerate() {
                let mark = if i == editor.pick_index { ">" } else { " " };
                out.push(format!("{mark} {}", def.name));
            }
            let new_mark = if editor.pick_index == count { ">" } else { " " };
            out.push(format!("{new_mark} (New dashboard…)"));
        }
        DashboardEditorScreen::NewFromPreset => {
            out.push("New from preset".into());
            out.push("j/k · Enter · Esc".into());
            for (i, label) in PRESET_LABELS.iter().enumerate() {
                let mark = if i == editor.preset_index { ">" } else { " " };
                out.push(format!("{mark} {label}"));
            }
        }
        DashboardEditorScreen::EditLayout => {
            let active = editor.set_as_active;
            let active_mark = if active { "[x]" } else { "[ ]" };
            let focus = editor.layout_focus;
            let rows_mark = if focus == DashboardEditLayoutFocus::Rows {
                ">"
            } else {
                " "
            };
            let cols_mark = if focus == DashboardEditLayoutFocus::Cols {
                ">"
            } else {
                " "
            };
            out.push(format!("Editing: {}", editor.draft.name));
            out.push(format!(
                "{rows_mark} rows: {} (+/-)   {cols_mark} cols: {} (+/-)",
                editor.draft.rows, editor.draft.cols
            ));
            out.push(format!(
                "{} Set as active dashboard",
                if focus == DashboardEditLayoutFocus::SetActive {
                    ">"
                } else {
                    " "
                }
            ));
            out.push(format!("{active_mark} active on save"));
            out.push("Panes (j/k · a add · d remove · e edit):".into());
            if editor.draft.panes.is_empty() {
                out.push("  (none)".into());
            }
            for (i, pane) in editor.draft.panes.iter().enumerate() {
                let mark =
                    if focus == DashboardEditLayoutFocus::PaneList && i == editor.selected_pane {
                        if editor.remove_armed {
                            "!"
                        } else {
                            ">"
                        }
                    } else {
                        " "
                    };
                out.push(format!(
                    "{mark} {} {} @{},{} {}×{}",
                    pane.id,
                    pane.kind.label(),
                    pane.row,
                    pane.col,
                    pane.row_span,
                    pane.col_span
                ));
            }
            let save_mark = if focus == DashboardEditLayoutFocus::Save {
                ">"
            } else {
                " "
            };
            let save_ok = validate_dashboard_definition(&editor.draft).is_ok();
            out.push(format!(
                "{save_mark} Save & close{}",
                if save_ok { "" } else { " (invalid)" }
            ));
            out.push("Tab: cycle focus · Space: toggle active · Esc: close".into());
        }
        DashboardEditorScreen::EditPane => {
            let Some(form) = &editor.pane_form else {
                out.push("Pane form missing".into());
                return out;
            };
            out.push(format!("Pane: {}", editor.draft.panes[form.pane_index].id));
            push_pane_field(
                &mut out,
                form,
                DashboardPaneFormField::Kind,
                "kind",
                form.kind.label(),
            );
            push_pane_field(
                &mut out,
                form,
                DashboardPaneFormField::Row,
                "row",
                &form.row_buf,
            );
            push_pane_field(
                &mut out,
                form,
                DashboardPaneFormField::Col,
                "col",
                &form.col_buf,
            );
            push_pane_field(
                &mut out,
                form,
                DashboardPaneFormField::RowSpan,
                "row_span",
                &form.row_span_buf,
            );
            push_pane_field(
                &mut out,
                form,
                DashboardPaneFormField::ColSpan,
                "col_span",
                &form.col_span_buf,
            );
            push_pane_field(
                &mut out,
                form,
                DashboardPaneFormField::Title,
                "title",
                &form.title_buf,
            );
            if form.kind == DashboardPaneKind::News {
                push_pane_field(
                    &mut out,
                    form,
                    DashboardPaneFormField::MaxRows,
                    "max_rows",
                    &form.max_rows_buf,
                );
            }
            out.push("c: next kind · Tab: field · type title · Enter: apply · Esc: back".into());
        }
        DashboardEditorScreen::ConfirmDiscard => {
            if editor.confirm_discard_return == Some(DashboardEditorScreen::EditPane) {
                out.push("Discard pane edits?".into());
            } else {
                out.push("Discard unsaved changes?".into());
            }
            out.push("y: discard · n: keep editing".into());
        }
    }
    if let Some(ref err) = editor.inline_error {
        out.push(String::new());
        out.push(format!("! {err}"));
    }
    out
}

fn push_pane_field(
    out: &mut Vec<String>,
    form: &DashboardPaneForm,
    field: DashboardPaneFormField,
    label: &str,
    value: &str,
) {
    let mark = if form.focused == field { ">" } else { " " };
    out.push(format!("{mark} {label}: {value}"));
}

/// Handle keys on the Dashboard tab (Issue #208 / §71).
pub fn handle_dashboard_events(app: &mut App, key: KeyEvent) {
    if app.dashboard_editor.is_some() {
        handle_dashboard_editor_keys(app, key);
        return;
    }
    if let Some(Action::DashboardOpenEditor) =
        app.resolved_keymap.action(BindingLayer::Dashboard, &key)
    {
        if letter_key_plain(key.modifiers) {
            app.open_dashboard_editor();
        }
    }
}

fn handle_dashboard_editor_keys(app: &mut App, key: KeyEvent) {
    let layer = BindingLayer::DashboardEditor;
    let action = app.resolved_keymap.action(layer, &key);

    if app
        .dashboard_editor
        .as_ref()
        .is_some_and(|e| e.screen == DashboardEditorScreen::ConfirmDiscard)
    {
        if letter_key_plain(key.modifiers) {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if app.dashboard_editor.as_ref().is_some_and(|e| {
                        e.confirm_discard_return == Some(DashboardEditorScreen::EditPane)
                    }) {
                        if let Some(editor) = app.dashboard_editor.as_mut() {
                            editor.pane_form = None;
                            editor.screen = DashboardEditorScreen::EditLayout;
                            editor.confirm_discard_return = None;
                        }
                        app.invalidate_dashboard_preview_caches();
                    } else {
                        app.close_dashboard_editor_discard();
                    }
                    app.refresh_dashboard_editor_overlay();
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    if let Some(editor) = app.dashboard_editor.as_mut() {
                        editor.screen = editor
                            .confirm_discard_return
                            .take()
                            .unwrap_or(DashboardEditorScreen::EditLayout);
                    }
                    app.refresh_dashboard_editor_overlay();
                }
                _ => {}
            }
        }
        if matches!(action, Some(Action::DashboardEditorEsc)) {
            if let Some(editor) = app.dashboard_editor.as_mut() {
                editor.screen = editor
                    .confirm_discard_return
                    .take()
                    .unwrap_or(DashboardEditorScreen::EditLayout);
                editor.confirm_discard_return = None;
            }
            app.refresh_dashboard_editor_overlay();
        }
        return;
    }

    if let Some(editor) = app.dashboard_editor.as_ref() {
        if editor.screen == DashboardEditorScreen::EditPane {
            handle_dashboard_editor_pane_form(app, key, action);
            app.refresh_dashboard_editor_overlay();
            return;
        }
    }

    match action {
        Some(Action::DashboardEditorEsc) => app.dashboard_editor_esc(),
        Some(Action::DashboardEditorSave) => {
            let enter = key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE;
            let ctrl_s =
                key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL);
            if enter {
                if app.dashboard_editor.as_ref().is_some_and(|e| {
                    matches!(
                        e.screen,
                        DashboardEditorScreen::PickDashboard | DashboardEditorScreen::NewFromPreset
                    )
                }) {
                    editor_pick_enter(app);
                    return;
                }
                if app.dashboard_editor.as_ref().is_some_and(|e| {
                    e.screen == DashboardEditorScreen::EditLayout
                        && e.layout_focus == DashboardEditLayoutFocus::Save
                }) {
                    app.commit_dashboard_editor();
                }
                return;
            }
            if ctrl_s {
                app.commit_dashboard_editor();
            }
        }
        Some(Action::DashboardEditorRowDown) if nav_key_ok(&key) => {
            editor_nav_down(app);
        }
        Some(Action::DashboardEditorRowUp) if nav_key_ok(&key) => {
            editor_nav_up(app);
        }
        Some(Action::DashboardEditorAddPane) if letter_key_plain(key.modifiers) => {
            editor_add_pane(app);
        }
        Some(Action::DashboardEditorRemoveArm) if letter_key_plain(key.modifiers) => {
            if key.code == KeyCode::Char('d') || key.code == KeyCode::Char('D') {
                if app
                    .dashboard_editor
                    .as_ref()
                    .is_some_and(|e| e.remove_armed)
                {
                    editor_remove_confirm(app);
                } else {
                    editor_remove_arm(app);
                }
            }
        }
        Some(Action::DashboardEditorEditPane) if letter_key_plain(key.modifiers) => {
            editor_open_pane_form(app);
        }
        Some(Action::DashboardEditorGridInc) => editor_grid_inc(app),
        Some(Action::DashboardEditorGridDec) => editor_grid_dec(app),
        Some(Action::DashboardEditorFocusNext) if tab_key_plain(key.modifiers) => {
            editor_cycle_layout_focus(app);
        }
        _ if key.code == KeyCode::Char(' ')
            && app
                .dashboard_editor
                .as_ref()
                .is_some_and(|e| e.screen == DashboardEditorScreen::EditLayout) =>
        {
            editor_toggle_active(app);
        }
        _ if letter_key_plain(key.modifiers)
            && app.dashboard_editor.as_ref().is_some_and(|e| {
                e.screen == DashboardEditorScreen::EditLayout && e.remove_armed
            }) =>
        {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => editor_remove_confirm(app),
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    if let Some(editor) = app.dashboard_editor.as_mut() {
                        editor.remove_armed = false;
                    }
                }
                _ => {}
            }
        }
        _ => {
            if let Some(editor) = app.dashboard_editor.as_mut() {
                if (editor.screen == DashboardEditorScreen::PickDashboard
                    || editor.screen == DashboardEditorScreen::NewFromPreset)
                    && key.code == KeyCode::Enter
                    && key.modifiers == KeyModifiers::NONE
                {
                    editor_pick_enter(app);
                }
            }
        }
    }
}

fn tab_key_plain(mods: KeyModifiers) -> bool {
    mods == KeyModifiers::NONE || mods == KeyModifiers::SHIFT
}

fn nav_key_ok(key: &KeyEvent) -> bool {
    letter_key_plain(key.modifiers) || key.modifiers == KeyModifiers::NONE
}

fn handle_dashboard_editor_pane_form(app: &mut App, key: KeyEvent, action: Option<Action>) {
    match action {
        Some(Action::DashboardEditorEsc) => {
            let discard_pane = app
                .dashboard_editor
                .as_ref()
                .is_some_and(|e| e.pane_form.as_ref().is_some_and(|f| f.dirty));
            if let Some(editor) = app.dashboard_editor.as_mut() {
                if discard_pane {
                    editor.confirm_discard_return = Some(DashboardEditorScreen::EditPane);
                    editor.screen = DashboardEditorScreen::ConfirmDiscard;
                } else {
                    editor.screen = DashboardEditorScreen::EditLayout;
                    editor.pane_form = None;
                }
            }
        }
        Some(Action::DashboardEditorSave) => {
            let enter = key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE;
            let ctrl_s =
                key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL);
            if enter || ctrl_s {
                editor_apply_pane_form(app);
            }
        }
        Some(Action::DashboardEditorKindNext)
            if letter_key_plain(key.modifiers)
                && app.dashboard_editor.as_ref().is_some_and(|e| {
                    e.pane_form
                        .as_ref()
                        .is_some_and(|f| f.focused == DashboardPaneFormField::Kind)
                }) =>
        {
            if let Some(form) = app
                .dashboard_editor
                .as_mut()
                .and_then(|e| e.pane_form.as_mut())
            {
                form.kind = form.kind.next();
                if form.kind != DashboardPaneKind::News {
                    form.max_rows_buf.clear();
                }
                form.dirty = true;
            }
        }
        Some(Action::DashboardEditorFocusNext) if tab_key_plain(key.modifiers) => {
            if let Some(form) = app
                .dashboard_editor
                .as_mut()
                .and_then(|e| e.pane_form.as_mut())
            {
                form.focused = pane_form_next_field(form.focused, form.kind);
            }
        }
        Some(Action::DashboardEditorBackspace) => {
            pane_form_backspace(app);
        }
        Some(Action::DashboardEditorDigit) => {
            pane_form_push_digit(app, key);
        }
        _ => {
            pane_form_append_title_char(app, key);
        }
    }
}

/// Append a printable character to the pane title buffer (Issue #208 / §71).
fn pane_form_append_title_char(app: &mut App, key: KeyEvent) {
    if !letter_key_plain(key.modifiers) {
        return;
    }
    let KeyCode::Char(ch) = key.code else {
        return;
    };
    if !is_pane_title_char(ch) {
        return;
    }
    let Some(form) = app
        .dashboard_editor
        .as_mut()
        .and_then(|e| e.pane_form.as_mut())
    else {
        return;
    };
    if form.focused != DashboardPaneFormField::Title {
        return;
    }
    const MAX_TITLE_LEN: usize = 48;
    if form.title_buf.len() < MAX_TITLE_LEN {
        form.title_buf.push(ch);
        form.dirty = true;
    }
}

fn is_pane_title_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '-' | '_' | '.' | '(' | ')')
}

fn pane_form_next_field(
    current: DashboardPaneFormField,
    kind: DashboardPaneKind,
) -> DashboardPaneFormField {
    use DashboardPaneFormField::*;
    let seq: &[DashboardPaneFormField] = if kind == DashboardPaneKind::News {
        &[Kind, Row, Col, RowSpan, ColSpan, Title, MaxRows]
    } else {
        &[Kind, Row, Col, RowSpan, ColSpan, Title]
    };
    let idx = seq.iter().position(|&f| f == current).unwrap_or(0);
    seq[(idx + 1) % seq.len()]
}

fn pane_form_backspace(app: &mut App) {
    let Some(form) = app
        .dashboard_editor
        .as_mut()
        .and_then(|e| e.pane_form.as_mut())
    else {
        return;
    };
    if form.focused == DashboardPaneFormField::Kind {
        return;
    }
    if let Some(buf) = pane_form_buffer_mut(form) {
        buf.pop();
        form.dirty = true;
    }
}

fn pane_form_push_digit(app: &mut App, key: KeyEvent) {
    let Some(ch) = digit_char(key) else {
        return;
    };
    let Some(form) = app
        .dashboard_editor
        .as_mut()
        .and_then(|e| e.pane_form.as_mut())
    else {
        return;
    };
    if form.focused == DashboardPaneFormField::Kind {
        return;
    }
    let Some(buf) = pane_form_buffer_mut(form) else {
        return;
    };
    if buf.len() < 2 {
        buf.push(ch);
        form.dirty = true;
    }
}

fn digit_char(key: KeyEvent) -> Option<char> {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => Some(c),
        _ => None,
    }
}

fn pane_form_buffer_mut(form: &mut DashboardPaneForm) -> Option<&mut String> {
    match form.focused {
        DashboardPaneFormField::Row => Some(&mut form.row_buf),
        DashboardPaneFormField::Col => Some(&mut form.col_buf),
        DashboardPaneFormField::RowSpan => Some(&mut form.row_span_buf),
        DashboardPaneFormField::ColSpan => Some(&mut form.col_span_buf),
        DashboardPaneFormField::Title => Some(&mut form.title_buf),
        DashboardPaneFormField::MaxRows => Some(&mut form.max_rows_buf),
        DashboardPaneFormField::Kind => None,
    }
}

fn editor_pick_enter(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    match editor.screen {
        DashboardEditorScreen::PickDashboard => {
            let count = app.config.dashboards.len();
            if editor.pick_index < count {
                let idx = editor.pick_index;
                let def = app.config.dashboards[idx].clone();
                let active = app.config.active_dashboard.as_deref();
                editor.editing_index = Some(idx);
                editor.draft = def;
                editor.dirty = false;
                editor.set_as_active = active == Some(editor.draft.name.as_str());
                editor.screen = DashboardEditorScreen::EditLayout;
                editor.refresh_validation_hint();
            } else {
                editor.screen = DashboardEditorScreen::NewFromPreset;
                editor.preset_index = 0;
                sync_preset_preview(editor, &app.config.dashboards);
            }
        }
        DashboardEditorScreen::NewFromPreset => {
            let preset_idx = editor
                .preset_index
                .min(PRESET_LABELS.len().saturating_sub(1));
            let name = allocate_dashboard_name(&app.config.dashboards);
            let preset = preset_by_index(preset_idx);
            editor.draft = clone_preset_with_name(&preset, &name);
            editor.editing_index = None;
            editor.dirty = true;
            editor.set_as_active = true;
            editor.screen = DashboardEditorScreen::EditLayout;
            editor.refresh_validation_hint();
        }
        _ => {}
    }
    app.invalidate_dashboard_preview_caches();
}

fn editor_nav_down(app: &mut App) {
    let mut refresh_preview = false;
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    match editor.screen {
        DashboardEditorScreen::PickDashboard => {
            let max = app.config.dashboards.len();
            editor.pick_index = (editor.pick_index + 1).min(max);
            sync_pick_preview(editor, &app.config);
            refresh_preview = true;
        }
        DashboardEditorScreen::NewFromPreset => {
            editor.preset_index = (editor.preset_index + 1).min(PRESET_LABELS.len() - 1);
            sync_preset_preview(editor, &app.config.dashboards);
            refresh_preview = true;
        }
        DashboardEditorScreen::EditLayout => {
            if editor.layout_focus == DashboardEditLayoutFocus::PaneList
                && !editor.draft.panes.is_empty()
            {
                editor.selected_pane = (editor.selected_pane + 1).min(editor.draft.panes.len() - 1);
            }
            editor.remove_armed = false;
        }
        _ => {}
    }
    if refresh_preview {
        app.invalidate_dashboard_preview_caches();
    }
}

fn editor_nav_up(app: &mut App) {
    let mut refresh_preview = false;
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    match editor.screen {
        DashboardEditorScreen::PickDashboard => {
            editor.pick_index = editor.pick_index.saturating_sub(1);
            sync_pick_preview(editor, &app.config);
            refresh_preview = true;
        }
        DashboardEditorScreen::NewFromPreset => {
            editor.preset_index = editor.preset_index.saturating_sub(1);
            sync_preset_preview(editor, &app.config.dashboards);
            refresh_preview = true;
        }
        DashboardEditorScreen::EditLayout => {
            if editor.layout_focus == DashboardEditLayoutFocus::PaneList {
                editor.selected_pane = editor.selected_pane.saturating_sub(1);
            }
            editor.remove_armed = false;
        }
        _ => {}
    }
    if refresh_preview {
        app.invalidate_dashboard_preview_caches();
    }
}

fn editor_cycle_layout_focus(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    if editor.screen != DashboardEditorScreen::EditLayout {
        return;
    }
    editor.layout_focus = match editor.layout_focus {
        DashboardEditLayoutFocus::Rows => DashboardEditLayoutFocus::Cols,
        DashboardEditLayoutFocus::Cols => DashboardEditLayoutFocus::PaneList,
        DashboardEditLayoutFocus::PaneList => DashboardEditLayoutFocus::SetActive,
        DashboardEditLayoutFocus::SetActive => DashboardEditLayoutFocus::Save,
        DashboardEditLayoutFocus::Save => DashboardEditLayoutFocus::Rows,
    };
    editor.remove_armed = false;
    app.refresh_dashboard_editor_overlay();
}

fn editor_toggle_active(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    if editor.screen == DashboardEditorScreen::EditLayout {
        editor.set_as_active = !editor.set_as_active;
        editor.mark_dirty();
        app.invalidate_dashboard_preview_caches();
    }
}

fn editor_grid_inc(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    if editor.screen != DashboardEditorScreen::EditLayout {
        return;
    }
    match editor.layout_focus {
        DashboardEditLayoutFocus::Rows if editor.draft.rows < 4 => {
            editor.draft.rows += 1;
            editor.mark_dirty();
        }
        DashboardEditLayoutFocus::Cols if editor.draft.cols < 4 => {
            editor.draft.cols += 1;
            editor.mark_dirty();
        }
        _ => return,
    }
    editor.refresh_validation_hint();
    app.invalidate_dashboard_preview_caches();
}

fn editor_grid_dec(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    if editor.screen != DashboardEditorScreen::EditLayout {
        return;
    }
    match editor.layout_focus {
        DashboardEditLayoutFocus::Rows if editor.draft.rows > 1 => {
            editor.draft.rows -= 1;
            editor.mark_dirty();
        }
        DashboardEditLayoutFocus::Cols if editor.draft.cols > 1 => {
            editor.draft.cols -= 1;
            editor.mark_dirty();
        }
        _ => return,
    }
    editor.refresh_validation_hint();
    app.invalidate_dashboard_preview_caches();
}

fn editor_add_pane(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    if editor.screen != DashboardEditorScreen::EditLayout {
        return;
    }
    let id = allocate_dashboard_pane_id(&editor.draft.panes);
    editor.draft.panes.push(DashboardPane {
        id,
        kind: DashboardPaneKind::Watchlist,
        row: 0,
        col: 0,
        row_span: 1,
        col_span: 1,
        title: None,
        options: DashboardPaneOptions::default(),
    });
    editor.selected_pane = editor.draft.panes.len().saturating_sub(1);
    editor.layout_focus = DashboardEditLayoutFocus::PaneList;
    editor.mark_dirty();
    editor.refresh_validation_hint();
    app.invalidate_dashboard_preview_caches();
}

fn editor_remove_arm(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    if editor.screen != DashboardEditorScreen::EditLayout
        || editor.draft.panes.is_empty()
        || editor.layout_focus != DashboardEditLayoutFocus::PaneList
    {
        return;
    }
    editor.remove_armed = !editor.remove_armed;
    app.refresh_dashboard_editor_overlay();
}

fn editor_remove_confirm(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    if editor.screen != DashboardEditorScreen::EditLayout || !editor.remove_armed {
        return;
    }
    let idx = editor
        .selected_pane
        .min(editor.draft.panes.len().saturating_sub(1));
    if idx < editor.draft.panes.len() {
        editor.draft.panes.remove(idx);
        editor.selected_pane = editor
            .selected_pane
            .min(editor.draft.panes.len().saturating_sub(1));
        editor.mark_dirty();
        editor.remove_armed = false;
        editor.refresh_validation_hint();
        app.invalidate_dashboard_preview_caches();
    }
}

fn editor_open_pane_form(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    if editor.screen != DashboardEditorScreen::EditLayout || editor.draft.panes.is_empty() {
        return;
    }
    let idx = editor.selected_pane.min(editor.draft.panes.len() - 1);
    let pane = &editor.draft.panes[idx];
    editor.pane_form = Some(DashboardPaneForm {
        pane_index: idx,
        kind: pane.kind,
        row_buf: pane.row.to_string(),
        col_buf: pane.col.to_string(),
        row_span_buf: pane.row_span.to_string(),
        col_span_buf: pane.col_span.to_string(),
        title_buf: pane.title.clone().unwrap_or_default(),
        max_rows_buf: pane
            .options
            .max_rows
            .map(|n| n.to_string())
            .unwrap_or_default(),
        focused: DashboardPaneFormField::Kind,
        dirty: false,
    });
    editor.screen = DashboardEditorScreen::EditPane;
    editor.remove_armed = false;
}

fn parse_u8_field(buf: &str, default: u8) -> Option<u8> {
    if buf.trim().is_empty() {
        return Some(default);
    }
    buf.trim().parse::<u8>().ok()
}

fn editor_apply_pane_form(app: &mut App) {
    let Some(editor) = app.dashboard_editor.as_mut() else {
        return;
    };
    let Some(form) = editor.pane_form.take() else {
        return;
    };
    let idx = form.pane_index;
    if idx >= editor.draft.panes.len() {
        editor.screen = DashboardEditorScreen::EditLayout;
        app.refresh_dashboard_editor_overlay();
        return;
    }
    let row = parse_u8_field(&form.row_buf, 0).unwrap_or(0);
    let col = parse_u8_field(&form.col_buf, 0).unwrap_or(0);
    let row_span = parse_u8_field(&form.row_span_buf, 1).unwrap_or(1);
    let col_span = parse_u8_field(&form.col_span_buf, 1).unwrap_or(1);
    let mut options = editor.draft.panes[idx].options.clone();
    if form.kind == DashboardPaneKind::News {
        let parsed = parse_u8_field(&form.max_rows_buf, 8).unwrap_or(8);
        options.max_rows = Some(crate::app::charts::clamp_dashboard_news_max_rows(Some(
            parsed,
        )));
    } else {
        options.max_rows = None;
    }
    let title = if form.title_buf.trim().is_empty() {
        None
    } else {
        Some(form.title_buf.trim().to_string())
    };
    let pane = DashboardPane {
        id: editor.draft.panes[idx].id.clone(),
        kind: form.kind,
        row,
        col,
        row_span,
        col_span,
        title,
        options,
    };
    if let Err(e) = validate_dashboard_pane(&editor.draft, &pane, Some(idx)) {
        editor.inline_error = Some(e.to_string());
        editor.pane_form = Some(form);
        editor.screen = DashboardEditorScreen::EditPane;
        app.refresh_dashboard_editor_overlay();
        return;
    }
    editor.draft.panes[idx] = pane;
    editor.screen = DashboardEditorScreen::EditLayout;
    editor.mark_dirty();
    editor.refresh_validation_hint();
    app.invalidate_dashboard_preview_caches();
}

impl App {
    /// Open the dashboard layout editor (Issue #208 / §71).
    pub fn open_dashboard_editor(&mut self) {
        self.dashboard_editor_config_snapshot = Some((
            self.config.dashboards.clone(),
            self.config.active_dashboard.clone(),
        ));
        let screen = if self.config.dashboards.is_empty() {
            DashboardEditorScreen::NewFromPreset
        } else {
            DashboardEditorScreen::PickDashboard
        };
        let mut editor = DashboardEditor {
            screen,
            draft: preset_dual_watchlist(),
            editing_index: None,
            dirty: false,
            selected_pane: 0,
            remove_armed: false,
            inline_error: None,
            pane_form: None,
            set_as_active: false,
            layout_focus: DashboardEditLayoutFocus::Rows,
            pick_index: 0,
            preset_index: 0,
            confirm_discard_return: None,
            new_dashboard_preview_name: None,
            overlay_lines: Vec::new(),
        };
        match editor.screen {
            DashboardEditorScreen::PickDashboard => {
                sync_pick_preview(&mut editor, &self.config);
                let active = self.config.active_dashboard.as_deref();
                editor.set_as_active = active == Some(editor.draft.name.as_str());
            }
            DashboardEditorScreen::NewFromPreset => {
                sync_preset_preview(&mut editor, &self.config.dashboards);
                editor.set_as_active = true;
            }
            _ => {}
        }
        self.dashboard_editor = Some(editor);
        self.invalidate_dashboard_preview_caches();
    }

    /// Discard editor without saving (Issue #208 / §71).
    pub fn close_dashboard_editor_discard(&mut self) {
        if let Some((dashboards, active)) = self.dashboard_editor_config_snapshot.take() {
            self.config.dashboards = dashboards;
            self.config.active_dashboard = active;
        }
        self.dashboard_editor = None;
        self.invalidate_dashboard_caches_after_config_change();
    }

    /// Clear editor when leaving Dashboard tab (§71.2).
    pub fn clear_dashboard_editor(&mut self) {
        if self.dashboard_editor.is_some() {
            self.close_dashboard_editor_discard();
        }
    }

    fn dashboard_editor_esc(&mut self) {
        let Some(editor) = &self.dashboard_editor else {
            return;
        };
        if editor.dirty {
            if let Some(e) = self.dashboard_editor.as_mut() {
                e.confirm_discard_return = None;
                e.screen = DashboardEditorScreen::ConfirmDiscard;
                self.refresh_dashboard_editor_overlay();
            }
        } else {
            self.dashboard_editor = None;
            self.dashboard_editor_config_snapshot = None;
            self.invalidate_dashboard_caches_after_config_change();
        }
    }

    /// Persist editor draft to config (Issue #208 / §71.6).
    pub fn commit_dashboard_editor(&mut self) {
        let Some(editor) = self.dashboard_editor.clone() else {
            return;
        };
        if editor.screen != DashboardEditorScreen::EditLayout {
            return;
        }
        let snapshot = self.dashboard_editor_config_snapshot.clone();
        let mut draft = editor.draft.clone();
        draft.name = draft.name.trim().to_string();
        if let Err(errs) = validate_dashboard_definition(&draft) {
            if let Some(e) = self.dashboard_editor.as_mut() {
                e.inline_error = Some(format_dashboard_validation_errors(&errs));
            }
            self.refresh_dashboard_editor_overlay();
            return;
        }
        if let Err(e) = validate_dashboard_name_unique(
            &draft.name,
            &self.config.dashboards,
            editor.editing_index,
        ) {
            if let Some(ed) = self.dashboard_editor.as_mut() {
                ed.inline_error = Some(e.to_string());
            }
            self.refresh_dashboard_editor_overlay();
            return;
        }
        let editing_index = editor.editing_index;
        let set_active = editor.set_as_active;

        match editing_index {
            Some(i) if i < self.config.dashboards.len() => {
                self.config.dashboards[i] = draft.clone();
            }
            _ => {
                self.config.dashboards.push(draft.clone());
            }
        }
        if set_active {
            self.config.active_dashboard = Some(draft.name.clone());
        }

        if let Err(e) = self.try_save_config_with_session() {
            if let Some((dashboards, active)) = snapshot {
                self.config.dashboards = dashboards;
                self.config.active_dashboard = active;
            }
            self.surface_runtime_error(
                Tab::Dashboard,
                ErrorSourceDomain::Dashboard,
                AppError::ConfigSave(format!("Failed to save dashboard: {e}")),
                true,
            );
            return;
        }

        self.dashboard_editor = None;
        self.dashboard_editor_config_snapshot = None;
        if self
            .active_runtime_error
            .as_ref()
            .is_some_and(|a| a.source_domain == ErrorSourceDomain::Dashboard)
        {
            self.active_runtime_error = None;
        }
        self.settings_saved_flash_until = Some(Instant::now() + Duration::from_secs(2));
        self.invalidate_dashboard_caches_after_config_change();
    }

    /// Rebuild dashboard caches after config or preview changes (§71.5).
    pub(crate) fn invalidate_dashboard_caches_after_config_change(&mut self) {
        self.dashboard_layout_cache = None;
        crate::app::dashboard_display::rebuild_dashboard_display_strings(self);
    }

    /// Rebuild preview caches while the editor is open (§71.5).
    pub(crate) fn invalidate_dashboard_preview_caches(&mut self) {
        self.invalidate_dashboard_caches_after_config_change();
        self.refresh_dashboard_editor_overlay();
    }

    /// Rebuild cached editor modal lines after state changes (Issue #208 / `rust_tui.mdc`).
    pub(crate) fn refresh_dashboard_editor_overlay(&mut self) {
        let Some(editor) = self.dashboard_editor.as_ref() else {
            return;
        };
        let lines = editor_lines(self, editor);
        if let Some(editor) = self.dashboard_editor.as_mut() {
            editor.overlay_lines = lines;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dashboard::{preset_dual_watchlist, preset_market_overview};

    #[test]
    fn pane_form_enter_applies_title() {
        let mut app = App::new();
        app.open_dashboard_editor();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.screen = DashboardEditorScreen::EditLayout;
            ed.draft = clone_preset_with_name(&preset_dual_watchlist(), "t");
            ed.draft.panes[0].title = None;
            ed.pane_form = Some(DashboardPaneForm {
                pane_index: 0,
                kind: DashboardPaneKind::Watchlist,
                row_buf: "0".into(),
                col_buf: "0".into(),
                row_span_buf: "1".into(),
                col_span_buf: "1".into(),
                title_buf: "Left".into(),
                max_rows_buf: String::new(),
                focused: DashboardPaneFormField::Title,
                dirty: false,
            });
            ed.screen = DashboardEditorScreen::EditPane;
        }
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        handle_dashboard_editor_keys(&mut app, enter);
        let ed = app.dashboard_editor.as_ref().unwrap();
        assert_eq!(ed.screen, DashboardEditorScreen::EditLayout);
        assert_eq!(ed.draft.panes[0].title.as_deref(), Some("Left"));
    }

    #[test]
    fn pane_form_kind_backspace_does_not_touch_title() {
        let mut app = App::new();
        app.open_dashboard_editor();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.screen = DashboardEditorScreen::EditPane;
            ed.pane_form = Some(DashboardPaneForm {
                pane_index: 0,
                kind: DashboardPaneKind::Watchlist,
                row_buf: "0".into(),
                col_buf: "0".into(),
                row_span_buf: "1".into(),
                col_span_buf: "1".into(),
                title_buf: "Keep".into(),
                max_rows_buf: String::new(),
                focused: DashboardPaneFormField::Kind,
                dirty: false,
            });
        }
        handle_dashboard_editor_keys(
            &mut app,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        );
        assert_eq!(
            app.dashboard_editor
                .as_ref()
                .unwrap()
                .pane_form
                .as_ref()
                .unwrap()
                .title_buf,
            "Keep"
        );
    }

    #[test]
    fn commit_dashboard_editor_persists_new_preset() {
        let mut app = App::new();
        app.config.dashboards.clear();
        app.config.active_dashboard = None;
        app.open_dashboard_editor();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.screen = DashboardEditorScreen::EditLayout;
            ed.draft = clone_preset_with_name(&preset_dual_watchlist(), "my_dash");
            ed.editing_index = None;
            ed.set_as_active = true;
            ed.dirty = true;
        }
        app.commit_dashboard_editor();
        assert!(app.dashboard_editor.is_none());
        assert_eq!(app.config.dashboards.len(), 1);
        assert_eq!(app.config.active_dashboard.as_deref(), Some("my_dash"));
    }

    #[test]
    fn pick_new_row_preview_name_is_stable() {
        let mut app = App::new();
        app.open_dashboard_editor();
        let count = app.config.dashboards.len();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.screen = DashboardEditorScreen::PickDashboard;
            ed.pick_index = count;
            sync_pick_preview(ed, &app.config);
        }
        let name_a = app.dashboard_editor.as_ref().unwrap().draft.name.clone();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.pick_index = count.saturating_sub(1);
            sync_pick_preview(ed, &app.config);
            ed.pick_index = count;
            sync_pick_preview(ed, &app.config);
        }
        assert_eq!(name_a, app.dashboard_editor.as_ref().unwrap().draft.name);
    }

    #[test]
    fn pick_navigation_updates_preview_draft() {
        let mut app = App::new();
        app.config.dashboards = vec![
            clone_preset_with_name(&preset_dual_watchlist(), "first"),
            clone_preset_with_name(&preset_market_overview(), "second"),
        ];
        app.open_dashboard_editor();
        assert_eq!(app.dashboard_editor.as_ref().unwrap().draft.name, "first");
        handle_dashboard_editor_keys(
            &mut app,
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
        );
        assert_eq!(app.dashboard_editor.as_ref().unwrap().draft.name, "second");
    }

    #[test]
    fn pane_form_ctrl_s_applies_title() {
        let mut app = App::new();
        app.open_dashboard_editor();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.screen = DashboardEditorScreen::EditLayout;
            ed.draft = clone_preset_with_name(&preset_dual_watchlist(), "t");
            ed.pane_form = Some(DashboardPaneForm {
                pane_index: 0,
                kind: DashboardPaneKind::Watchlist,
                row_buf: "0".into(),
                col_buf: "0".into(),
                row_span_buf: "1".into(),
                col_span_buf: "1".into(),
                title_buf: "Ctrl".into(),
                max_rows_buf: String::new(),
                focused: DashboardPaneFormField::Title,
                dirty: false,
            });
            ed.screen = DashboardEditorScreen::EditPane;
        }
        handle_dashboard_editor_keys(
            &mut app,
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
        );
        assert_eq!(
            app.dashboard_editor.as_ref().unwrap().draft.panes[0]
                .title
                .as_deref(),
            Some("Ctrl")
        );
    }

    #[test]
    fn pane_form_esc_dirty_prompts_confirm() {
        let mut app = App::new();
        app.open_dashboard_editor();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.screen = DashboardEditorScreen::EditPane;
            ed.pane_form = Some(DashboardPaneForm {
                pane_index: 0,
                kind: DashboardPaneKind::Watchlist,
                row_buf: "0".into(),
                col_buf: "0".into(),
                row_span_buf: "1".into(),
                col_span_buf: "1".into(),
                title_buf: "x".into(),
                max_rows_buf: String::new(),
                focused: DashboardPaneFormField::Title,
                dirty: true,
            });
        }
        handle_dashboard_editor_keys(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(
            app.dashboard_editor.as_ref().unwrap().screen,
            DashboardEditorScreen::ConfirmDiscard
        );
    }

    #[test]
    fn commit_validation_error_refreshes_overlay() {
        let mut app = App::new();
        app.open_dashboard_editor();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.screen = DashboardEditorScreen::EditLayout;
            ed.draft = clone_preset_with_name(&preset_dual_watchlist(), "t");
            ed.draft.panes.clear();
            ed.overlay_lines.clear();
        }
        app.commit_dashboard_editor();
        let ed = app.dashboard_editor.as_ref().unwrap();
        assert!(ed.inline_error.is_some());
        assert!(
            ed.overlay_lines.iter().any(|line| line.starts_with("! ")),
            "overlay should show validation error immediately"
        );
    }

    #[test]
    fn commit_stale_editing_index_appends_dashboard() {
        let mut app = App::new();
        app.config.dashboards.clear();
        app.open_dashboard_editor();
        {
            let ed = app.dashboard_editor.as_mut().unwrap();
            ed.screen = DashboardEditorScreen::EditLayout;
            ed.draft = clone_preset_with_name(&preset_dual_watchlist(), "orphan");
            ed.editing_index = Some(99);
            ed.set_as_active = false;
            ed.dirty = true;
        }
        app.commit_dashboard_editor();
        assert_eq!(app.config.dashboards.len(), 1);
        assert_eq!(app.config.dashboards[0].name, "orphan");
    }
}
