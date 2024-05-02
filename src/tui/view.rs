use super::model::App;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, HighlightSpacing, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

impl<'a> App<'a> {
    pub fn view(&mut self, frame: &mut Frame) {
        if !self.editing {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(frame.size());

            {
                let chunks =
                    Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(chunks[0]);

                self.view_search_bar(frame, chunks[0]);
                self.view_snippets_list(frame, chunks[1]);
            }
            self.view_snippet_details(frame, chunks[1]);
        } else {
            self.view_editor(frame, frame.size());
        }
    }

    fn view_search_bar(&mut self, frame: &mut Frame, rect: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(2));
        let inner = block.inner(rect);

        let search_bar = &mut self.search_bar;
        search_bar.set_placeholder_text("Type to search");

        frame.render_widget(block, rect);
        frame.render_widget(search_bar.widget(), inner);
    }

    fn view_snippets_list(&mut self, frame: &mut Frame, rect: Rect) {
        let block = Block::bordered().border_type(BorderType::Rounded);
        let inner = block.inner(rect);

        frame.render_widget(block, rect);

        if self.snippets.is_empty() {
            let nothing = Paragraph::new(
                Line::from("Empty ＞︿＜. Press `Ctrl-A` to add a new snippet ヾ(•ω•`)o").bold(),
            )
            .centered()
            .wrap(Wrap { trim: true });
            frame.render_widget(nothing, inner);
        } else {
            let items: Vec<ListItem> = self
                .snippets
                .iter()
                .enumerate()
                .map(|(index, snippet)| {
                    let line = Line::from(format!("{:02} {}", index, snippet.cmd));
                    ListItem::new(line)
                })
                .collect();

            let list = List::new(items)
                .highlight_symbol("🥢")
                .highlight_spacing(HighlightSpacing::Always)
                .highlight_style(Style::new().cyan().italic().bold());

            frame.render_stateful_widget(list, inner, &mut self.state);
        }
    }

    fn view_snippet_details(&mut self, frame: &mut Frame, rect: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(2));
        let inner = block.inner(rect);

        frame.render_widget(block, rect);

        let index = self.state.selected().unwrap();

        if let Some(snippet) = self.snippets.get(index) {
            let mut content = Text::default();

            content.push_line("[Command]".cyan().bold());
            content.extend(snippet.cmd.lines());
            content.push_line("[Description]".cyan().bold());
            content.extend(snippet.description.lines());

            let content = Paragraph::new(content).wrap(Wrap { trim: true });

            frame.render_widget(content, inner);
        } else {
            let nothing =
                Paragraph::new(Line::from("There's nothing. Let's select one, and the details will be displayed here OwO.").bold())
                    .centered()
                    .wrap(Wrap { trim: true });
            frame.render_widget(nothing, inner);
        }
    }

    fn view_editor(&mut self, frame: &mut Frame, rect: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1));
        let inner = block.inner(rect);
        // This is safe. Every time when open editor, it will be constructed.
        let editor = self.editor.as_mut().unwrap();
        editor.set_placeholder_text(
            "priority = 0\ncmd = \"echo hello world\"\ndescription = \"this is a example\"",
        );

        frame.render_widget(block, rect);
        frame.render_widget(editor.widget(), inner);
    }
}
