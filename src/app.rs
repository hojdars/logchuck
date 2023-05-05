use chrono::ParseError;
use log::*;
use std::{
    cmp::min,
    collections::{HashSet, VecDeque},
    io,
    path::Path,
    str::FromStr,
    time::Duration,
};

use futures::executor::block_on;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

use crate::timestamp::LineError;

use super::mergeline::merge;
use super::mergeline::Line;
use super::text::FileWithLines;

struct Common {
    items: VecDeque<String>,
    state: ListState,
    absolute_index: usize,
}

impl Common {
    fn new(it: Vec<String>) -> Common {
        Common {
            items: it.into(),
            state: ListState::default(),
            absolute_index: 0,
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
    fn new(files: Vec<FileWithLines>) -> Result<ViewMenu, LineError> {
        let mut res = ViewMenu {
            files,
            all_lines: Vec::new(),
        };

        for i in 0..res.files.len() {
            let file_lines: Vec<Line> = res.files[i].get_annotated_lines(i)?;
            res.all_lines = merge(&res.all_lines, &file_lines);
        }

        info!(
            "ViewMenu::new - lines merged, total count={}",
            res.all_lines.len()
        );

        Ok(res)
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
    error: Option<String>,
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
            error: None,
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
        match &mut self.app_state {
            AppState::FileList(_) => {
                let mut i = self.common.state.selected().unwrap();
                if i >= self.common.items.len() - 1 {
                    i = 0
                } else {
                    i += 1
                }
                self.common.state.select(Some(i));
            }
            AppState::TextView(view) => {
                let mut i = self.common.state.selected().unwrap();
                if i == self.common.items.len() - 1 {
                    let first_not_loaded = self.common.absolute_index + 1;
                    let new_lines = view.get_lines(first_not_loaded, first_not_loaded + 1);
                    if new_lines.is_empty() {
                        return; // no wrap
                    } else {
                        assert_eq!(new_lines.len(), 1);
                        self.common.items.push_back(new_lines[0].clone());
                        self.common.items.pop_front();
                        self.common.absolute_index += 1;
                    }
                } else {
                    i += 1;
                    self.common.absolute_index += 1;
                }
                self.common.state.select(Some(i));
            }
        }
        assert!(self.common.items.len() <= self.terminal_size.height as usize);
    }

    fn select_previous(&mut self) {
        match &mut self.app_state {
            AppState::FileList(_) => {
                let mut i = self.common.state.selected().unwrap();
                if i == 0 {
                    i = self.common.items.len() - 1;
                } else {
                    i -= 1;
                }
                self.common.state.select(Some(i));
            }
            AppState::TextView(view) => {
                let mut i = self.common.state.selected().unwrap();
                if i == 0 {
                    if self.common.absolute_index == 0 {
                        return; // no wrap
                    } else {
                        let new_lines = view
                            .get_lines(self.common.absolute_index - 1, self.common.absolute_index);
                        assert_eq!(new_lines.len(), 1);
                        self.common.items.pop_back();
                        self.common.items.push_front(new_lines[0].clone());
                        self.common.absolute_index = self.common.absolute_index.saturating_sub(1);
                    }
                } else {
                    i -= 1;
                    self.common.absolute_index = self.common.absolute_index.saturating_sub(1);
                }
                self.common.state.select(Some(i));
            }
        }
        assert!(self.common.items.len() <= self.terminal_size.height as usize);
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

                self.app_state = match ViewMenu::new(files) {
                    Ok(new_state) => AppState::TextView(new_state),
                    Err(err) => {
                        self.error = Some(format!(
                            "App::load_files - cannot load files, error={}",
                            err
                        ));
                        return;
                    }
                };

                if let AppState::TextView(view) = &self.app_state {
                    if !view.files.is_empty() {
                        self.common.items =
                            view.get_lines(0, self.terminal_size.height.into()).into();
                        self.common.state = ListState::default();
                        self.common.absolute_index = 0;
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
        self.common.items = self.file_list.clone().into();
        self.common.state = ListState::default();

        if !self.common.items.is_empty() {
            self.common.state.select(Some(0));
        }
    }

    fn page_down(&mut self) {
        match &mut self.app_state {
            AppState::TextView(view) => {
                let new_from = min(
                    view.all_lines.len(),
                    self.common.absolute_index + self.terminal_size.height as usize / 2,
                );
                let new_end = new_from + self.terminal_size.height as usize;
                let mut new_items = view.get_lines(new_from, new_end);
                if new_items.is_empty() {
                    return;
                }
                while new_items.len() < self.terminal_size.height as usize {
                    new_items.push("~".to_string());
                }
                self.common.items = new_items.into();
                self.common.absolute_index = new_from;
                self.common.state.select(Some(min(
                    self.common.state.selected().unwrap(),
                    self.common.items.len(),
                )));
            }
            AppState::FileList(_) => {}
        }
    }

    fn page_up(&mut self) {
        // TODO: Bug - Doing 'End' (or scrolling to the end) and then pressing Page-Up jumps wrongly.
        match &mut self.app_state {
            AppState::TextView(view) => {
                let new_from = self
                    .common
                    .absolute_index
                    .saturating_sub(self.terminal_size.height as usize / 2);
                let new_end = new_from + self.terminal_size.height as usize;
                let mut new_items = view.get_lines(new_from, new_end);
                if new_items.is_empty() {
                    return;
                }
                while new_items.len() < self.terminal_size.height as usize {
                    new_items.push("~".to_string());
                }
                self.common.items = new_items.into();
                self.common.absolute_index = new_from;
                self.common.state.select(Some(min(
                    self.common.state.selected().unwrap(),
                    self.common.items.len(),
                )));
            }
            AppState::FileList(_) => {}
        }
    }

    fn home(&mut self) {
        match &mut self.app_state {
            AppState::TextView(view) => {
                let mut new_items = view.get_lines(0, self.terminal_size.height as usize);
                while new_items.len() < self.terminal_size.height as usize {
                    new_items.push("~".to_string());
                }
                self.common.items = new_items.into();
                self.common.absolute_index = 0;
                self.common.state.select(Some(0));
            }
            AppState::FileList(_) => {}
        }
    }

    fn end(&mut self) {
        match &mut self.app_state {
            AppState::TextView(view) => {
                let new_to: usize = view.all_lines.len();
                let new_from: usize = new_to.saturating_sub(self.terminal_size.height as usize);
                let mut new_items = view.get_lines(new_from, new_to);
                while new_items.len() < self.terminal_size.height as usize {
                    new_items.push("~".to_string());
                }
                self.common.items = new_items.into();
                self.common.absolute_index = new_to - 1;
                self.common.state.select(Some(self.common.items.len() - 1));
            }
            AppState::FileList(_) => {}
        }
    }

    fn clear_popup(&mut self) {
        self.error = None
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    // solarized: https://ethanschoonover.com/solarized/
    let fg_color = Color::Rgb(147, 161, 161);
    let bg_color = Color::Rgb(0, 43, 54);
    let fg_accent_color = Color::Rgb(181, 137, 0);
    let bg_accent_color = Color::Rgb(7, 54, 66);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)].as_ref())
        .split(size);

    let bl = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(bg_color).fg(fg_color));
    f.render_widget(bl, chunks[0]);

    let mid_menu_row = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)].as_ref())
        .split(chunks[0]);
    assert_eq!(mid_menu_row.len(), 2);

    let mid_menu_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(10), Constraint::Min(0)].as_ref())
        .split(mid_menu_row[1]);
    assert_eq!(mid_menu_center.len(), 2);

    let titles_text: Vec<String> = vec!["Files".to_string(), "Log".to_string()];
    let titles: Vec<Spans> = titles_text
        .iter()
        .map(|t| {
            Spans::from(Span::styled(
                t,
                Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let selected_tab: usize = match app.app_state {
        AppState::FileList(_) => 0,
        AppState::TextView(_) => 1,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::NONE))
        .select(selected_tab)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .highlight_style(Style::default().bg(bg_accent_color).fg(fg_accent_color));

    f.render_widget(tabs, mid_menu_center[1]);

    app.terminal_size = chunks[1];

    let list_items: Vec<ListItem> = match &mut app.app_state {
        AppState::FileList(file_list) => app
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
                    String::from_str(Path::new(i).file_name().unwrap().to_str().unwrap()).unwrap()
                )))
                .style(Style::default().fg(fg_color).bg(bg_color))
            })
            .collect(),
        AppState::TextView(_) => app
            .common
            .items
            .iter()
            .map(|i| {
                ListItem::new(Span::from(String::from(i)))
                    .style(Style::default().fg(fg_color).bg(bg_color))
            })
            .collect(),
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .style(Style::default().bg(bg_color)),
        )
        .highlight_style(Style::default().fg(fg_accent_color).bg(bg_accent_color));

    f.render_stateful_widget(list, chunks[1], &mut app.common.state);

    if let Some(error_text) = &app.error {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        f.render_widget(tui::widgets::Clear, area); //this clears out the background

        let text = error_text.to_owned() + "\n\nPress 'Esc' to close this popup";

        let paragraph = Paragraph::new(text.clone())
            .style(Style::default().bg(bg_accent_color).fg(fg_accent_color))
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
    }
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
                    crossterm::event::KeyCode::Char(' ') => app.flip_current(),
                    crossterm::event::KeyCode::Enter => app.load_files(),
                    crossterm::event::KeyCode::Backspace => app.go_to_file_list(),
                    crossterm::event::KeyCode::PageUp => app.page_up(),
                    crossterm::event::KeyCode::PageDown => app.page_down(),
                    crossterm::event::KeyCode::Home => app.home(),
                    crossterm::event::KeyCode::End => app.end(),
                    crossterm::event::KeyCode::Esc => app.clear_popup(),
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

/// Taken from TUI examples: https://github.com/fdehau/tui-rs/blob/v0.19.0/examples/popup.rs
/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
