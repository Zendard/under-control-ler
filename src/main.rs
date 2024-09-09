use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, List, ListState, Paragraph, StatefulWidget, Widget,
    },
    DefaultTerminal,
};
use std::io;

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

#[derive(Debug, Default)]
pub struct HostConfig {
    port: String,
    character_index: usize,
}

#[derive(Debug, Default)]
pub struct JoinConfig {
    address: String,
    port: String,
    character_index: usize,
    current_input: JoinInput,
}

#[derive(Debug, Default)]
pub enum JoinInput {
    #[default]
    Address,
    Port,
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
        let AppState::Host(ref mut host_config) = self.state else {
            return;
        };
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.prev_state(),
            KeyCode::Enter => self.join(),
            KeyCode::Char(to_insert) => host_config.enter_char(to_insert),
            KeyCode::Backspace => host_config.delete_char(),
            KeyCode::Left => host_config.move_cursor_left(),
            KeyCode::Right => host_config.move_cursor_right(),
            _ => {}
        }
    }

    fn handel_key_event_join(&mut self, key_event: KeyEvent) {
        let AppState::Join(ref mut join_config) = self.state else {
            return;
        };
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.prev_state(),
            KeyCode::Enter => self.join(),
            KeyCode::Down => join_config.next_input(),
            KeyCode::Up => join_config.prev_input(),
            KeyCode::Char(to_insert) => join_config.enter_char(to_insert),
            KeyCode::Backspace => join_config.delete_char(),
            KeyCode::Left => join_config.move_cursor_left(),
            KeyCode::Right => join_config.move_cursor_right(),
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

    fn join(&mut self) {
        let AppState::Join(ref join_config) = self.state else {
            return;
        };

        under_control_ler::join(under_control_ler::JoinConfig {
            address: join_config.address.clone(),
            port: join_config.port.clone(),
        });
    }
}

impl JoinConfig {
    fn prev_input(&mut self) {
        match self.current_input {
            JoinInput::Address => {}
            JoinInput::Port => self.current_input = JoinInput::Address,
        }
    }

    fn next_input(&mut self) {
        match self.current_input {
            JoinInput::Address => self.current_input = JoinInput::Port,
            JoinInput::Port => {}
        }
    }

    fn enter_char(&mut self, to_insert: char) {
        let index = self.byte_index();
        let input_field = match self.current_input {
            JoinInput::Address => &mut self.address,
            JoinInput::Port => &mut self.port,
        };

        input_field.insert(index, to_insert);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        let input_field = match self.current_input {
            JoinInput::Address => &self.address,
            JoinInput::Port => &self.port,
        };

        input_field
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(input_field.len())
    }

    fn delete_char(&mut self) {
        if self.character_index == 0 {
            return;
        }

        let input_field = match self.current_input {
            JoinInput::Address => &mut self.address,
            JoinInput::Port => &mut self.port,
        };

        let from_left_to_current_index = self.character_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = input_field.chars().take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = input_field.chars().skip(self.character_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        *input_field = before_char_to_delete.chain(after_char_to_delete).collect();
        self.move_cursor_left();
    }

    fn move_cursor_left(&mut self) {
        let input_field = match self.current_input {
            JoinInput::Address => &mut self.address,
            JoinInput::Port => &mut self.port,
        };

        let cursor_moved_right = self.character_index.saturating_sub(1);
        self.character_index = cursor_moved_right.clamp(0, input_field.chars().count())
    }

    fn move_cursor_right(&mut self) {
        let input_field = match self.current_input {
            JoinInput::Address => &mut self.address,
            JoinInput::Port => &mut self.port,
        };

        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = cursor_moved_right.clamp(0, input_field.chars().count())
    }
}

impl HostConfig {
    fn enter_char(&mut self, to_insert: char) {
        let index = self.byte_index();
        let input_field = &mut self.port;

        input_field.insert(index, to_insert);
        self.move_cursor_right();
    }

    fn byte_index(&mut self) -> usize {
        let input_field = &mut self.port;

        input_field
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(input_field.len())
    }

    fn delete_char(&mut self) {
        if self.character_index == 0 {
            return;
        }

        let input_field = &mut self.port;

        let from_left_to_current_index = self.character_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = input_field.chars().take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = input_field.chars().skip(self.character_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        *input_field = before_char_to_delete.chain(after_char_to_delete).collect();
        self.move_cursor_left();
    }

    fn move_cursor_left(&mut self) {
        let input_field = &mut self.port;

        let cursor_moved_right = self.character_index.saturating_sub(1);
        self.character_index = cursor_moved_right.clamp(0, input_field.chars().count())
    }

    fn move_cursor_right(&mut self) {
        let input_field = &mut self.port;

        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = cursor_moved_right.clamp(0, input_field.chars().count())
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
            Some(0) => *self = AppState::Host(HostConfig::default()),
            Some(1) => *self = AppState::Join(JoinConfig::default()),
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

        let main_area = center(area, Constraint::Length(30), Constraint::Length(3));

        Block::bordered()
            .title(Title::from("Enter port"))
            .render(main_area, buf);

        let byte_index = host_config.byte_index();
        let mut port_text = host_config.port.clone();
        port_text.insert(byte_index, '▌');

        let port_input = Paragraph::new(port_text).block(Block::bordered().title("Port:"));

        port_input.render(main_area, buf)
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

        Block::bordered()
            .title(Title::from("Enter address and port"))
            .render(main_area, buf);

        let [address_area, port_area] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(main_area);

        let byte_index = join_config.byte_index();

        let mut address_text = join_config.address.clone();
        let mut port_text = join_config.port.clone();
        match join_config.current_input {
            JoinInput::Address => address_text.insert(byte_index, '▌'),
            JoinInput::Port => port_text.insert(byte_index, '▌'),
        };

        let (address_input, port_input) = App::input_widgets(address_text, port_text);

        address_input.render(address_area, buf);
        port_input.render(port_area, buf)
    }

    fn input_widgets(
        address_text: String,
        port_text: String,
    ) -> (Paragraph<'static>, Paragraph<'static>) {
        let address_input = Paragraph::new(address_text).block(Block::bordered().title("Address:"));
        let port_input = Paragraph::new(port_text).block(Block::bordered().title("Port:"));

        (address_input, port_input)
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
