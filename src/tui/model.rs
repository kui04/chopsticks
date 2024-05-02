use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Result;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use tui_textarea::TextArea;

use crate::event::EventHandler;
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug, Default)]
pub struct Snippet {
    #[serde(default)]
    pub priority: i64,
    pub cmd: String,
    pub description: String,
}

impl ToString for Snippet {
    #[rustfmt::skip]
    fn to_string(&self) -> String {
        format!(
            "priority = {}\ncmd = \'\'\'{}\'\'\'\ndescription = \'\'\'{}\'\'\'",
            self.priority,
            if self.cmd.is_empty() { "\n" } else { self.cmd.as_str() },
            if self.description.is_empty() { "\n" } else { self.cmd.as_str() },
        )
    }
}

#[derive(Debug)]
pub struct App<'a> {
    pub quit: bool,
    pub(super) editing: bool,
    pub(super) search_bar: TextArea<'a>,
    pub(super) editor: Option<TextArea<'a>>,
    pub(super) snippets: Vec<Snippet>,
    pub(super) state: ListState,
    pub(super) events: EventHandler,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        Self {
            quit: false,
            editing: false,
            search_bar: TextArea::default(),
            editor: None,
            snippets: Vec::new(),
            state: ListState::default(),
            events: EventHandler::new(16),
        }
    }

    pub fn init(&mut self) {
        self.snippets = self.load_snippets().expect("Failed to load snippets file");
        self.state.select(Some(0));
    }

    pub fn quit(&mut self) {
        let snippets =
            toml::to_string_pretty(&HashMap::from([("snippets", &self.snippets)])).unwrap();
        let snippet_path = Self::snippet_path();
        fs::write(snippet_path, snippets).unwrap();
        self.quit = true;
    }

    fn load_snippets(&mut self) -> Result<Vec<Snippet>> {
        let snippet_path = Self::snippet_path();
        let content = if snippet_path.exists() {
            fs::read_to_string(snippet_path)?
        } else {
            fs::create_dir_all(snippet_path.parent().unwrap())?;
            fs::File::create(snippet_path)?;
            String::new()
        };

        let mut toml = toml::from_str::<HashMap<String, Vec<Snippet>>>(&content)?;

        Ok(toml
            .remove("snippets")
            .get_or_insert_with(Vec::new)
            .to_owned())
    }

    fn snippet_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap()
            .join("chopsticks")
            .join("snippets.toml")
    }
}
