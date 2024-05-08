use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use run_script::{types::ScriptOptions, IoOptions};

use crate::{event::Event, tui::model::Snippet};

use super::{model::App, restore_terminal};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Msg {
    AppClose,
    SelectNext,
    SelectPrev,
    ExecuteCmd,
    RemoveSnippet,
    Edit(EditMsg),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EditMsg {
    Open { snippet: Snippet },
    Cancel,
    Save,
}

impl<'a> App<'a> {
    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::SelectNext => self.select_next(),
            Msg::SelectPrev => self.select_previous(),
            Msg::RemoveSnippet => self.remove_snippet(),
            Msg::ExecuteCmd => {
                self.execute_cmd().unwrap();
                self.quit()
            }
            Msg::Edit(EditMsg::Open { snippet }) => {
                self.editing = true;
                self.editor = Some(snippet.to_string().lines().collect());
            }
            Msg::Edit(EditMsg::Save) => {
                self.save_snippet();
                self.editing = false;
                self.editor = None;
            }
            Msg::Edit(EditMsg::Cancel) => {
                self.editing = false;
                self.editor = None;
            }
            Msg::AppClose => self.quit(),
        }
    }

    pub async fn handle_event(&mut self) -> Option<Msg> {
        match self.events.next().await? {
            Event::Key(key_evt) => {
                if self.editing {
                    self.handle_edit_event(key_evt)
                } else {
                    match (key_evt.code, key_evt.modifiers) {
                        // Exit application on `ESC` or `Ctrl-C`
                        (KeyCode::Esc, _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL)
                        | (KeyCode::Char('C'), KeyModifiers::CONTROL) => Some(Msg::AppClose),
                        _ => self.handle_key_event(key_evt),
                    }
                }
            }

            Event::Mouse(mouse_evt) => self.handle_mouse_event(mouse_evt),
            Event::Tick => None,
        }
    }

    fn handle_key_event(&mut self, evt: KeyEvent) -> Option<Msg> {
        match (evt.code, evt.modifiers) {
            (KeyCode::Up, _) => Some(Msg::SelectPrev),
            (KeyCode::Down, _) => Some(Msg::SelectNext),
            (KeyCode::Enter, _) => Some(Msg::ExecuteCmd),
            (KeyCode::Char('a'), KeyModifiers::CONTROL)
            | (KeyCode::Char('A'), KeyModifiers::CONTROL) => Some(Msg::Edit(EditMsg::Open {
                snippet: Snippet::default(),
            })),
            (KeyCode::Char('e'), KeyModifiers::CONTROL)
            | (KeyCode::Char('E'), KeyModifiers::CONTROL) => {
                if self.snippets.is_empty() {
                    None
                } else {
                    let index = self.state.selected().unwrap();
                    let snippet = self.snippets.remove(index);
                    Some(Msg::Edit(EditMsg::Open { snippet }))
                }
            }
            (KeyCode::Char('r'), KeyModifiers::CONTROL)
            | (KeyCode::Char('R'), KeyModifiers::CONTROL) => Some(Msg::RemoveSnippet),

            _ => {
                self.search_bar.input(evt);
                self.search_snippet();
                None
            }
        }
    }

    fn handle_edit_event(&mut self, evt: KeyEvent) -> Option<Msg> {
        match (evt.code, evt.modifiers) {
            (KeyCode::Char('s'), KeyModifiers::CONTROL)
            | (KeyCode::Char('S'), KeyModifiers::CONTROL) => Some(Msg::Edit(EditMsg::Save)),

            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('C'), KeyModifiers::CONTROL) => Some(Msg::Edit(EditMsg::Cancel)),

            _ => {
                self.editor.as_mut().unwrap().input(evt);
                None
            }
        }
    }

    fn handle_mouse_event(&self, evt: MouseEvent) -> Option<Msg> {
        match evt.kind {
            MouseEventKind::ScrollDown => Some(Msg::SelectNext),
            MouseEventKind::ScrollUp => Some(Msg::SelectPrev),
            _ => None,
        }
    }

    fn select_next(&mut self) {
        // This won't panic because 'selected' is initialized to 0 from the beginning.
        let i = self.state.selected().unwrap();
        let i = if i >= self.snippets.len().saturating_sub(1) {
            0
        } else {
            i + 1
        };

        self.state.select(Some(i));
    }

    fn select_previous(&mut self) {
        // This won't panic because 'selected' is initialized to 0 from the beginning.
        let i = self.state.selected().unwrap();
        let i = if i == 0 {
            self.snippets.len().saturating_sub(1)
        } else {
            i - 1
        };
        self.state.select(Some(i));
    }

    fn execute_cmd(&self) -> Result<()> {
        let index = self.state.selected().unwrap();
        if let Some(snippet) = self.snippets.get(index) {
            let cmd = snippet.cmd.as_str();
            let mut options = ScriptOptions::new();
            options.output_redirection = IoOptions::Inherit;

            restore_terminal()?;
            self.events.stop();

            let _exit_status: std::process::ExitStatus =
                run_script::spawn_script!(cmd, &options)?.wait()?;
        }

        Ok(())
    }

    fn search_snippet(&mut self) {
        let matcher = SkimMatcherV2::default();

        self.snippets.iter_mut().for_each(|s| {
            let mut priority = 0i64;
            self.search_bar.lines()[0]
                .split_ascii_whitespace()
                .into_iter()
                .for_each(|k| {
                    priority += matcher.fuzzy_match(&s.cmd, k).unwrap_or_default();
                    priority += matcher.fuzzy_match(&s.description, k).unwrap_or_default();
                });
            s.priority = priority;
        });

        self.snippets.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.state.select(Some(0));
    }

    fn save_snippet(&mut self) {
        // TODO: popup err msg
        let snippet = self.editor.as_ref().unwrap().lines().join("\n");
        let snippet: Snippet = toml::from_str(&snippet).unwrap();
        self.snippets.push(snippet);
    }

    fn remove_snippet(&mut self) {
        let index = self.state.selected().unwrap();
        if !self.snippets.is_empty() {
            self.snippets.remove(index);
        }
    }
}
