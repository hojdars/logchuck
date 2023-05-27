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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};

use crate::timestamp::LineError;

use super::mergeline::merge;
use super::mergeline::Line;
use super::text::FileWithLines;

// solarized: https://ethanschoonover.com/solarized/
const FG_COLOR: Color = Color::Rgb(147, 161, 161);
const BG_COLOR: Color = Color::Rgb(0, 43, 54);
const FG_ACCENT_COLOR: Color = Color::Rgb(181, 137, 0);
const BG_ACCENT_COLOR: Color = Color::Rgb(7, 54, 66);
const ERROR_RED_COLOR: Color = Color::Rgb(220, 50, 47);
const WARN_YELLOW_COLOR: Color = Color::Rgb(181, 137, 0);

#[derive(Clone)]
struct FileEntry {
    filename: String,
    file_size: u64,
}

impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.filename.cmp(&other.filename)
    }
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FileEntry {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
    }
}

impl Eq for FileEntry {}

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
    details_dialog: Option<String>,
}

impl ViewMenu {
    fn new(files: Vec<FileWithLines>) -> Result<ViewMenu, LineError> {
        let mut res = ViewMenu {
            files,
            all_lines: Vec::new(),
            details_dialog: None,
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
                    .unwrap()
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

struct AppError {
    error_message: String,
}

struct App {
    common: Common,
    app_state: AppState,
    file_list: Vec<FileEntry>,
    terminal_size: tui::layout::Rect,
    error: Option<String>,
}

impl App {
    fn new(path: &std::path::Path, size: tui::layout::Rect) -> Result<App, std::io::Error> {
        info!("App::new - new App");
        let file_list = App::scan_directory(path)?;
        let mut app = App {
            common: Common::new(file_list.iter().map(|f| f.filename.clone()).collect()),
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

    fn scan_directory(path: &std::path::Path) -> Result<Vec<FileEntry>, std::io::Error> {
        let mut result: Vec<FileEntry> = Vec::new();
        for item in std::fs::read_dir(path)? {
            let item_path = item?.path();
            if !item_path.is_file() || item_path.file_name().is_none() {
                continue;
            }
            let metadata = std::fs::metadata(item_path.clone())?;
            match item_path.file_name().unwrap().to_os_string().into_string() {
                Ok(file) => {
                    if file.as_bytes()[0] != b'.' {
                        let fullpath = path
                            .join(std::path::Path::new(&file))
                            .canonicalize()
                            .unwrap();

                        if metadata.len() == 0 {
                            continue;
                        }
                        result.push(FileEntry {
                            filename: fullpath.as_os_str().to_os_string().into_string().unwrap(),
                            file_size: metadata.len(),
                        });
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

    fn enter(&mut self) {
        let result_new_state: Result<Option<AppState>, AppError> = match &mut self.app_state {
            AppState::TextView(view) => App::show_details_dialog(
                view,
                self.common.items[self.common.state.selected().unwrap()].clone(),
            ),
            AppState::FileList(file_list) => {
                if file_list.loaded_items.is_empty() {
                    return;
                } else {
                    App::load_files(file_list)
                }
            }
        };

        match result_new_state {
            Ok(new_state_opt) => {
                if let Some(new_state) = new_state_opt {
                    self.app_state = new_state;
                    match &self.app_state {
                        AppState::FileList(_) => {}
                        AppState::TextView(view) => {
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
            Err(error) => {
                self.error = Some(error.error_message);
            }
        }
    }

    fn load_files(file_list: &mut FileListMenu) -> Result<Option<AppState>, AppError> {
        let mut to_load: Vec<String> = Vec::new();
        for lf in &file_list.loaded_items {
            to_load.push(lf.clone());
        }

        info!("App::load_files - preparing to load files");
        let file_futures = Box::pin(FileWithLines::from_files(to_load));
        let files = block_on(file_futures);

        info!("App::load_files - {} files loaded", files.len());

        match ViewMenu::new(files) {
            Ok(new_state) => Ok(Some(AppState::TextView(new_state))),
            Err(err) => Err(AppError {
                error_message: format!("App::load_files - cannot load files, error={}", err),
            }),
        }
    }

    fn show_details_dialog(
        menu: &mut ViewMenu,
        text: String,
    ) -> Result<Option<AppState>, AppError> {
        match menu.details_dialog {
            Some(_) => menu.details_dialog = None,
            None => menu.details_dialog = Some(text),
        }
        return Ok(None);
    }

    fn go_to_file_list(&mut self) {
        match &self.app_state {
            AppState::FileList(_) => {}
            AppState::TextView(view_menu) => {
                let mut new_file_menu = FileListMenu::new();
                for file in &view_menu.files {
                    new_file_menu.loaded_items.insert(file.filename());
                }

                self.app_state = AppState::FileList(new_file_menu);

                self.common.items = self.file_list.iter().map(|f| f.filename.clone()).collect();
                self.common.state = ListState::default();

                if !self.common.items.is_empty() {
                    self.common.state.select(Some(0));
                }
            }
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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)].as_ref())
        .split(size);

    let bl = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(BG_COLOR).fg(FG_COLOR));
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
                Style::default().fg(FG_COLOR).add_modifier(Modifier::BOLD),
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
        .style(Style::default().bg(BG_COLOR).fg(FG_COLOR))
        .highlight_style(Style::default().bg(BG_ACCENT_COLOR).fg(FG_ACCENT_COLOR));

    f.render_widget(tabs, mid_menu_center[1]);

    app.terminal_size = chunks[1];

    let list_items: Vec<ListItem> = match &mut app.app_state {
        AppState::FileList(file_list) => generate_file_list(&app.file_list, file_list),
        AppState::TextView(_) => app
            .common
            .items
            .iter()
            .map(|i| {
                let style: Style;
                let line = String::from(i);
                if line.contains("ERROR") {
                    style = Style::default().fg(ERROR_RED_COLOR).bg(BG_COLOR);
                } else if line.contains("WARN") {
                    style = Style::default().fg(WARN_YELLOW_COLOR).bg(BG_COLOR);
                } else {
                    style = Style::default().fg(FG_COLOR).bg(BG_COLOR);
                }
                ListItem::new(Span::from(line)).style(style)
            })
            .collect(),
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .style(Style::default().bg(BG_COLOR)),
        )
        .highlight_style(Style::default().fg(FG_ACCENT_COLOR).bg(BG_ACCENT_COLOR));

    f.render_stateful_widget(list, chunks[1], &mut app.common.state);

    if let Some(error_text) = &app.error {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        f.render_widget(tui::widgets::Clear, area); //this clears out the background

        let text = error_text.to_owned() + "\n\nPress 'Esc' to close this popup";

        let paragraph = Paragraph::new(text)
            .style(Style::default().bg(BG_ACCENT_COLOR).fg(FG_ACCENT_COLOR))
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    match &mut app.app_state {
        AppState::FileList(_) => {}
        AppState::TextView(view) => {
            if let Some(text) = &view.details_dialog {
                let block = Block::default().title("Popup").borders(Borders::ALL);
                let area = centered_rect(60, 20, size);
                f.render_widget(tui::widgets::Clear, area); //this clears out the background

                let paragraph = Paragraph::new(text.clone())
                    .style(Style::default().bg(BG_ACCENT_COLOR).fg(FG_ACCENT_COLOR))
                    .block(block)
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false });
                f.render_widget(paragraph, area);
            }
        }
    }
}

fn generate_file_list<'a>(
    app_file_list: &'a Vec<FileEntry>,
    file_list: &FileListMenu,
) -> Vec<ListItem<'a>> {
    let mut max_filename_len: usize = 0;
    for f in app_file_list {
        let filename_string: String = String::from_str(
            Path::new(&f.filename)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();

        if filename_string.len() > max_filename_len {
            max_filename_len = filename_string.len();
        }
    }

    max_filename_len += 5;

    app_file_list
        .iter()
        .map(|i| {
            let loaded_marker = if file_list
                .loaded_items
                .contains(&App::to_abs_path(&i.filename))
            {
                "x"
            } else {
                " "
            };
            let mut filename_string = String::from_str(
                Path::new(&i.filename)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )
            .unwrap();
            if filename_string.len() < max_filename_len {
                filename_string +=
                    String::from_utf8(vec![b' '; max_filename_len - filename_string.len()])
                        .unwrap()
                        .as_str();
            }
            ListItem::new(Span::from(format!(
                "[{}] {} ({} B)",
                loaded_marker, filename_string, i.file_size
            )))
            .style(Style::default().fg(FG_COLOR).bg(BG_COLOR))
        })
        .collect()
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
                    crossterm::event::KeyCode::Down => app.select_next(),
                    crossterm::event::KeyCode::Up => app.select_previous(),
                    crossterm::event::KeyCode::Char(' ') => app.flip_current(),
                    crossterm::event::KeyCode::Enter => app.enter(),
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
