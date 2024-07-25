use std::{collections::HashMap, fmt::Display, fs, path::PathBuf};
use anyhow::Result;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{ListState, Paragraph, Widget},
};
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

impl Display for Snippet {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "priority = {}\ncmd = \'\'\'{}\'\'\'\ndescription = \'\'\'{}\'\'\'",
            self.priority,
            if self.cmd.is_empty() { "\n" } else { self.cmd.as_str() },
            if self.description.is_empty() { "\n" } else { self.description.as_str() },
        )
    }
}

#[derive(Debug)]
pub struct App<'a> {
    pub quit: bool,
    pub terminal_restored: bool,
    pub(super) is_editing: bool,
    pub(super) error_msg: Option<String>,
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

#[derive(Debug)]
pub struct TextAreaWidget<'a> {
    text_area: &'a TextArea<'a>,
}

impl<'a> TextAreaWidget<'a> {
    pub fn new(text_area: &'a TextArea<'a>) -> Self {
        Self { text_area }
    }
}

impl<'a> Widget for TextAreaWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let text = self.text_area.lines().join("\n");
        let paragraph = Paragraph::new(text);
        paragraph.render(area, buf);

        let (x, y) = self.text_area.cursor();

        // 修正光标的位置
        let cursor_x = area.left() + y as u16;
        let cursor_y = area.top() + x as u16;
        
        // 确保光标位置在渲染区域内
        if cursor_x >= area.left() && cursor_x <= area.right() && cursor_y >= area.top() && cursor_y < area.bottom() {
            buf.set_string(cursor_x, cursor_y, "|", Style::default());
        }
    }
}


impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        Self {
            quit: false,
            terminal_restored: false,
            is_editing: false,
            error_msg: None,
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

    pub fn quit(&mut self) -> Result<()> {
        let snippets =
            toml::to_string_pretty(&HashMap::from([("snippets", &self.snippets)])).unwrap();
        let snippet_path = Self::snippet_path();
        fs::write(snippet_path, snippets).unwrap();
        self.quit = true;
        Ok(())
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
