use std::{collections::HashSet, io, path::Path, time::Duration};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

struct App {
    items: Vec<String>,
    state: ListState,
    loaded_items: HashSet<String>,
}

impl App {
    fn new(path: &std::path::Path) -> Result<App, std::io::Error> {
        let mut app = App {
            items: App::scan_directory(path)?,
            state: ListState::default(),
            loaded_items: HashSet::new(),
        };

        if !app.items.is_empty() {
            app.state.select(Some(0));
        }

        Ok(app)
    }

    fn scan_directory(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
        let mut result: Vec<String> = Vec::new();
        for item in std::fs::read_dir(path)? {
            match item?.file_name().into_string() {
                Ok(file) => {
                    if file.as_bytes()[0] != b'.' {
                        result.push(file)
                    }
                }
                Err(err_file) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("filename is not Unicode, filename={:?}", err_file),
                    ))
                }
            }
        }

        result.sort();
        Ok(result)
    }

    fn select_next(&mut self) {
        let state = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => panic!("no file selected, this should not happen"),
        };
        self.state.select(Some(state));
    }

    fn select_previous(&mut self) {
        let state = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => panic!("no file selected, this should not happen"),
        };
        self.state.select(Some(state));
    }

    fn flip_current(&mut self) {
        if let Some(selected) = self.state.selected() {
            if self.loaded_items.contains(&self.items[selected]) {
                self.loaded_items.remove(&self.items[selected]);
            } else {
                assert!(selected < self.items.len());
                self.loaded_items.insert(self.items[selected].clone());
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let paragraph = Paragraph::new("\nLogfiles | Logs | Settings")
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, chunks[0]);

    let list_items: Vec<ListItem> = app
        .items
        .iter()
        .map(|i| {
            let loaded_marker = if app.loaded_items.contains(i) {
                "x"
            } else {
                " "
            };
            ListItem::new(Span::from(format!("[{}] {}", loaded_marker, i)))
                .style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(Color::White)),
        )
        .highlight_style(Style::default().bg(Color::LightBlue));

    f.render_stateful_widget(list, chunks[1], &mut app.state);
}

pub fn run_app(folder_to_run: &String) -> Result<(), io::Error> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app: App = App::new(Path::new(folder_to_run))?;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => break,
                    crossterm::event::KeyCode::Char('j') => app.select_next(),
                    crossterm::event::KeyCode::Char('k') => app.select_previous(),
                    crossterm::event::KeyCode::Down => app.select_next(),
                    crossterm::event::KeyCode::Up => app.select_previous(),
                    crossterm::event::KeyCode::Enter => app.flip_current(),
                    _ => {}
                }
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
