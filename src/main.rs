use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, List, ListState, StatefulWidget, Widget,
    },
    DefaultTerminal,
};

const SELECTED_STYLE: Style = Style::new()
    .bg(Color::White)
    .fg(Color::Black)
    .add_modifier(Modifier::BOLD);

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug)]
pub struct App {
    state: AppState,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            state: AppState::ModeSelect(ListState::default()),
            exit: false,
        }
    }
}

#[derive(Debug)]
pub enum AppState {
    ModeSelect(ListState),
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;

        if !matches!(
            event,
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                ..
            })
        ) {
            return Ok(());
        }

        let Event::Key(event) = event else {
            return Ok(());
        };

        match self.state {
            AppState::ModeSelect(_) => self.handle_key_event_mode_select(event),
        }
        Ok(())
    }

    fn handle_key_event_mode_select(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down | KeyCode::Char('j') => self.state.next_list_item(),
            KeyCode::Up | KeyCode::Char('k') => self.state.prev_list_item(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        dbg!(&self);
        self.exit = true;
    }
}

impl AppState {
    fn next_list_item(&mut self) {
        let AppState::ModeSelect(list_state) = self;
        list_state.select_next()
    }

    fn prev_list_item(&mut self) {
        let AppState::ModeSelect(list_state) = self;
        list_state.select_previous()
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &mut self.state {
            AppState::ModeSelect(list_state) => App::render_mode_select(area, buf, list_state),
        }
    }
}

impl App {
    fn render_mode_select(area: Rect, buf: &mut Buffer, list_state: &mut ListState) {
        let title = Title::from(" Under Control(ler) ".bold());
        let instructions = Title::from(Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        StatefulWidget::render(
            List::new(["Host", "Join"])
                .block(block)
                .highlight_style(SELECTED_STYLE),
            area,
            buf,
            list_state,
        );
    }
}
