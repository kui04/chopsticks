use super::model::App;
use crate::tui::model::TextAreaWidget;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, HighlightSpacing, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

impl<'a> App<'a> {
    pub fn view(&mut self, frame: &mut Frame) {
        let chunks =
            Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(frame.size());

        if self.is_editing {
            self.view_editor(frame, frame.size());
        } else {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[0]);

            {
                let chunks =
                    Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(chunks[0]);
                self.view_search_bar(frame, chunks[0]);
                self.view_snippets_list(frame, chunks[1]);
            }

            self.view_snippet_details(frame, chunks[1]);
        }

        if self.error_msg.is_some() {
            self.view_error_msg(frame, chunks[1]);
        } else {
            self.view_instructions(frame, chunks[1]);
        }
    }

    fn view_search_bar(&mut self, frame: &mut Frame, rect: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(2));
        let inner = block.inner(rect);

        let search_bar = &self.search_bar;
        let text_area_widget = TextAreaWidget::new(search_bar);

        frame.render_widget(block, rect);
        frame.render_widget(text_area_widget, inner);
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
        let text_area_widget = TextAreaWidget::new(editor);
        frame.render_widget(text_area_widget, inner);
    }

    fn view_instructions(&mut self, frame: &mut Frame, rect: Rect) {
        let inner = Block::new().padding(Padding::horizontal(1)).inner(rect);
        let instructions = Line::from(vec![
            Span::from("<Enter> Execute").bold().on_cyan(),
            Span::from(" | "),
            Span::from("Ctrl").bold().on_cyan(),
            Span::from(" + "),
            Span::from("<a> Add").on_dark_gray(),
            Span::from(" "),
            Span::from("<r> Remove").on_dark_gray(),
            Span::from(" "),
            Span::from("<e> Edit").on_dark_gray(),
            Span::from(" "),
            Span::from("<s> Save").on_dark_gray(),
            Span::from(" "),
            Span::from("<c> quit or cancel").on_dark_gray(),
            Span::from(" "),
            Span::from("<enter> copy").on_dark_gray(),
        ])
        .white()
        .alignment(Alignment::Left);

        frame.render_widget(instructions, inner);
    }

    fn view_error_msg(&self, frame: &mut Frame, rect: Rect) {
        let inner = Block::new().padding(Padding::horizontal(1)).inner(rect);
        let msg = self.error_msg.as_ref().unwrap();
        let content = Line::from(msg.as_str()).red();

        frame.render_widget(content, inner);
    }
}
