use log::*;
use std::{cmp::min, collections::HashSet, io, path::Path, str::FromStr, time::Duration};

use futures::executor::block_on;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use super::mergeline::merge;
use super::mergeline::Line;
use super::text::FileWithLines;

struct Common {
    items: Vec<String>,
    state: ListState,
}

impl Common {
    fn new(it: Vec<String>) -> Common {
        Common {
            items: it,
            state: ListState::default(),
        }
    }
}

struct FileListMenu {
    loaded_items: HashSet<String>,
}

impl FileListMenu {
    fn new() -> FileListMenu {
        FileListMenu {
            loaded_items: HashSet::new(),
        }
    }
}

struct ViewMenu {
    files: Vec<FileWithLines>,
    all_lines: Vec<Line>,
}

impl ViewMenu {
    fn new(files: Vec<FileWithLines>) -> ViewMenu {
        let mut res = ViewMenu {
            files,
            all_lines: Vec::new(),
        };

        for i in 0..res.files.len() {
            let file_lines: Vec<Line> = res.files[i].get_annotated_lines(i);
            res.all_lines = merge(&res.all_lines, &file_lines);
        }

        info!(
            "ViewMenu::new - lines merged, total count={}",
            res.all_lines.len()
        );

        res
    }

    fn get_lines(&self, from: usize, to: usize) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();

        let from = min(from, self.all_lines.len());
        let to = min(to, self.all_lines.len());

        if to <= from || from > self.all_lines.len() - 1 {
            return Vec::new();
        }

        for i in from..to {
            let line = &self.all_lines[i];
            result.push(
                self.files[line.source_file]
                    .get_ith_line(line.index)
                    .to_string(),
            );
        }

        result
    }
}

enum AppState {
    FileList(FileListMenu),
    TextView(ViewMenu),
}

struct App {
    common: Common,
    app_state: AppState,
    file_list: Vec<String>,
    terminal_size: tui::layout::Rect,
}

impl App {
    fn new(path: &std::path::Path, size: tui::layout::Rect) -> Result<App, std::io::Error> {
        info!("App::new - new App");
        let file_list = App::scan_directory(path)?;
        let mut app = App {
            common: Common::new(file_list.clone()),
            app_state: AppState::FileList(FileListMenu::new()),
            file_list,
            terminal_size: size,
        };

        if !app.common.items.is_empty() {
            app.common.state.select(Some(0));
        }

        Ok(app)
    }

    fn scan_directory(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
        let mut result: Vec<String> = Vec::new();
        for item in std::fs::read_dir(path)? {
            let item_path = item?.path();
            if !item_path.is_file() || item_path.file_name().is_none() {
                continue;
            }
            match item_path.file_name().unwrap().to_os_string().into_string() {
                Ok(file) => {
                    if file.as_bytes()[0] != b'.' {
                        let fullpath = path
                            .join(std::path::Path::new(&file))
                            .canonicalize()
                            .unwrap();
                        result.push(fullpath.as_os_str().to_os_string().into_string().unwrap())
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
        let state = match self.common.state.selected() {
            Some(i) => {
                if i >= self.common.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => panic!("no file selected, this should not happen"),
        };
        self.common.state.select(Some(state));
    }

    fn select_previous(&mut self) {
        let state = match self.common.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.common.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => panic!("no file selected, this should not happen"),
        };
        self.common.state.select(Some(state));
    }

    fn flip_current(&mut self) {
        match &mut self.app_state {
            AppState::FileList(file_list) => {
                if let Some(selected) = self.common.state.selected() {
                    if file_list
                        .loaded_items
                        .contains(&App::to_abs_path(&self.common.items[selected]))
                    {
                        file_list
                            .loaded_items
                            .remove(&App::to_abs_path(&self.common.items[selected]));
                    } else {
                        assert!(selected < self.common.items.len());
                        file_list
                            .loaded_items
                            .insert(App::to_abs_path(&self.common.items[selected]));
                    }
                }
            }
            AppState::TextView(_) => {}
        }
    }

    fn to_abs_path(path: &String) -> String {
        std::fs::canonicalize(path)
            .unwrap()
            .as_os_str()
            .to_os_string()
            .into_string()
            .unwrap()
    }

    fn load_files(&mut self) {
        match &mut self.app_state {
            AppState::TextView(_) => {}
            AppState::FileList(file_list) => {
                if file_list.loaded_items.is_empty() {
                    return;
                }

                let mut to_load: Vec<String> = Vec::new();
                for lf in &file_list.loaded_items {
                    to_load.push(lf.clone());
                }

                info!("App::load_files - preparing to load files");
                let file_futures = Box::pin(FileWithLines::from_files(to_load));
                let files = block_on(file_futures);

                info!("App::load_files - {} files loaded", files.len());

                self.app_state = AppState::TextView(ViewMenu::new(files));

                if let AppState::TextView(view) = &self.app_state {
                    if !view.files.is_empty() {
                        self.common.items = view.get_lines(0, self.terminal_size.height as usize);
                        self.common.state = ListState::default();

                        if !self.common.items.is_empty() {
                            self.common.state.select(Some(0));
                        }
                    }
                }
            }
        }
    }

    fn go_to_file_list(&mut self) {
        self.app_state = AppState::FileList(FileListMenu::new());
        self.common.items = self.file_list.clone();
        self.common.state = ListState::default();

        if !self.common.items.is_empty() {
            self.common.state.select(Some(0));
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

    let mut list_items: Vec<ListItem> = Vec::new();

    match &mut app.app_state {
        AppState::FileList(file_list) => {
            list_items = app
                .common
                .items
                .iter()
                .map(|i| {
                    let loaded_marker = if file_list.loaded_items.contains(&App::to_abs_path(i)) {
                        "x"
                    } else {
                        " "
                    };
                    ListItem::new(Span::from(format!(
                        "[{}] {}",
                        loaded_marker,
                        String::from_str(Path::new(i).file_name().unwrap().to_str().unwrap())
                            .unwrap()
                    )))
                    .style(Style::default().fg(Color::Black).bg(Color::White))
                })
                .collect();
        }
        AppState::TextView(_) => {
            list_items = app
                .common
                .items
                .iter()
                .map(|i| {
                    ListItem::new(Span::from(String::from(i)))
                        .style(Style::default().fg(Color::Black).bg(Color::White))
                })
                .collect();
        }
    }

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(Color::White)),
        )
        .highlight_style(Style::default().bg(Color::LightBlue));

    f.render_stateful_widget(list, chunks[1], &mut app.common.state);
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

    let mut app: App = App::new(Path::new(folder_to_run), terminal.size()?)?;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => break,
                    crossterm::event::KeyCode::Char('j') => app.select_next(),
                    crossterm::event::KeyCode::Char('k') => app.select_previous(),
                    crossterm::event::KeyCode::Char('g') => app.load_files(),
                    crossterm::event::KeyCode::Down => app.select_next(),
                    crossterm::event::KeyCode::Up => app.select_previous(),
                    crossterm::event::KeyCode::Enter => app.flip_current(),
                    crossterm::event::KeyCode::Backspace => app.go_to_file_list(),
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
