use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
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
    Host(HostConfig),
    Join(JoinConfig),
}

#[derive(Debug)]
pub struct HostConfig {
    address: SocketAddr,
}

#[derive(Debug)]
pub struct JoinConfig {
    address: SocketAddr,
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
            AppState::Host(_) => self.handel_key_event_host(event),
            AppState::Join(_) => self.handel_key_event_join(event),
        }
        Ok(())
    }

    fn handle_key_event_mode_select(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down | KeyCode::Char('j') => self.state.next_list_item(),
            KeyCode::Up | KeyCode::Char('k') => self.state.prev_list_item(),
            KeyCode::Enter => self.state.select_item(),
            KeyCode::Esc => self.prev_state(),
            _ => {}
        }
    }

    fn handel_key_event_host(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.prev_state(),
            _ => {}
        }
    }

    fn handel_key_event_join(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.prev_state(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        dbg!(&self);
        self.exit = true;
    }

    fn prev_state(&mut self) {
        let target_state = match self.state {
            AppState::Host(_) | AppState::Join(_) => AppState::ModeSelect(ListState::default()),
            _ => return,
        };

        self.state = target_state;
    }
}

impl AppState {
    fn next_list_item(&mut self) {
        if let AppState::ModeSelect(list_state) = self {
            list_state.select_next()
        }
    }

    fn prev_list_item(&mut self) {
        if let AppState::ModeSelect(list_state) = self {
            list_state.select_previous()
        }
    }

    fn select_item(&mut self) {
        let AppState::ModeSelect(list_state) = self else {
            return;
        };
        match list_state.selected() {
            Some(0) => {
                *self = AppState::Host(HostConfig {
                    address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8629)),
                })
            }
            Some(1) => {
                *self = AppState::Join(JoinConfig {
                    address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8629)),
                })
            }
            _ => {}
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &mut self.state {
            AppState::ModeSelect(list_state) => App::render_mode_select(area, buf, list_state),
            AppState::Host(host_config) => App::render_host(area, buf, host_config),
            AppState::Join(join_config) => App::render_join(area, buf, join_config),
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
        block.render(area, buf);

        let main_area = center(area, Constraint::Length(30), Constraint::Length(6));

        let main_block = Block::bordered()
            .title(Title::from(" Select mode "))
            .title_bottom(Line::from(vec![
                " Select item ".into(),
                "<↑↓> + <Enter> ".blue().bold(),
            ]));

        StatefulWidget::render(
            List::new(["Host", "Join"])
                .block(main_block)
                .highlight_style(SELECTED_STYLE),
            main_area,
            buf,
            list_state,
        );
    }

    fn render_host(area: Rect, buf: &mut Buffer, host_config: &mut HostConfig) {
        let title = Title::from(" Under Control(ler) ".bold());
        let instructions = Title::from(Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
            " Return ".into(),
            "<Esc> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);
        block.render(area, buf);

        let main_area = center(area, Constraint::Length(30), Constraint::Length(6));

        let main_block =
            Block::bordered().title(Title::from(format!("Hosting at {}", host_config.address)));
        main_block.render(main_area, buf)
    }

    fn render_join(area: Rect, buf: &mut Buffer, join_config: &mut JoinConfig) {
        let title = Title::from(" Under Control(ler) ".bold());
        let instructions = Title::from(Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
            " Return ".into(),
            "<Esc> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);
        block.render(area, buf);

        let main_area = center(area, Constraint::Length(30), Constraint::Length(6));

        let main_block =
            Block::bordered().title(Title::from(format!("Joining {}", join_config.address)));
        main_block.render(main_area, buf)
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
