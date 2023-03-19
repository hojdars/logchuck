use std::{collections::HashSet, env, io, path::Path, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use std::fs::File;
use std::mem::size_of;
use std::str::from_utf8;
use std::time::Instant;
use std::{fs, result, vec};

use futures::executor::block_on;
use tokio::io::AsyncReadExt;
use tokio::join;
use tokio::task::JoinSet;

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

fn read_file_to_string(path: &String) -> String {
    fs::read_to_string(path).expect("Should have been able to read the file")
}

async fn async_read_to_string(path: String) -> Vec<String> {
    let res = tokio::fs::read_to_string(path).await.unwrap();
    println!("load done");
    let r = res.split('\n');
    println!("split done");
    vec![res]
}

async fn file_run() {
    let mut futures: JoinSet<Vec<String>> = JoinSet::new();

    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        buffer.retain(|c| !c.is_whitespace());

        if buffer == "done" {
            break;
        }

        let path = Path::new(&buffer);

        if path.exists() {
            futures.spawn(async_read_to_string(buffer.clone()));
            // read_file_to_string(buffer.clone());
        }
    }
    println!("file input done, start reading");

    while let Some(result) = futures.join_next().await {
        let text: Vec<String> = result.unwrap();
        println!("#lines: {}", text.len());
    }
}

fn read_and_merge() -> Vec<String> {
    const BUFFER_SIZE: usize = 500000;

    let mut left_file = std::fs::File::open("testdata\\left.log").unwrap();
    let mut right_file = std::fs::File::open("testdata\\right.log").unwrap();

    let mut result: Vec<String> = Vec::new();

    let mut left_rest: Option<String> = None;
    let mut right_rest: Option<String> = None;

    loop {
        let mut left_buffer = [0; BUFFER_SIZE];
        let left_ret = left_file.read(&mut left_buffer[..]);
        let mut right_buffer = [0; BUFFER_SIZE];
        let right_ret = right_file.read(&mut right_buffer[..]);

        let (result_left, result_right) = (left_ret, right_ret);

        if let Err(e) = result_right {
            panic!("{}", e);
        }
        if let Err(e) = result_left {
            panic!("{}", e);
        }
        let (result_left, result_right) = (result_left.unwrap(), result_right.unwrap());

        if result_left == 0 && result_right == 0 {
            return result;
        } else if result_left == 0 {
            let right_str: String = from_utf8(&right_buffer[0..result_right])
                .unwrap()
                .to_string();
            let mut right_iter = right_str.split_inclusive('\n');
            while let Some(item) = right_iter.next() {
                let mut item: String = item.to_string();
                if let Some(rest) = right_rest {
                    item = rest + &item;
                    right_rest = None;
                }

                if !item.is_empty() && item.chars().last().unwrap() == '\n' {
                    result.push(item.to_string());
                } else if !item.is_empty() {
                    right_rest = Some(item.to_string());
                }
            }
        } else if result_right == 0 {
            let left_str: String = from_utf8(&left_buffer[0..result_left]).unwrap().to_string();
            let mut left_iter = left_str.split_inclusive('\n');
            while let Some(item) = left_iter.next() {
                let mut item: String = item.to_string();
                if let Some(rest) = left_rest {
                    item = rest + &item;
                    left_rest = None;
                }

                if !item.is_empty() && item.chars().last().unwrap() == '\n' {
                    result.push(item.to_string());
                } else if !item.is_empty() {
                    left_rest = Some(item.to_string());
                }
            }
        } else {
            let left_str: String = from_utf8(&left_buffer[0..result_left]).unwrap().to_string();
            let right_str: String = from_utf8(&right_buffer[0..result_right])
                .unwrap()
                .to_string();

            let mut left_iter = left_str.split_inclusive('\n');
            let mut right_iter = right_str.split_inclusive('\n');

            let mut i: usize = 0;
            let mut left_empty: bool = false;
            let mut right_empty: bool = false;
            loop {
                if i % 2 == 0 {
                    if let Some(item) = left_iter.next() {
                        let mut item: String = item.to_string();
                        if let Some(rest) = left_rest {
                            item = rest + &item;
                            left_rest = None;
                        }

                        if !item.is_empty() && item.chars().last().unwrap() == '\n' {
                            result.push(item.to_string());
                        } else if !item.is_empty() {
                            left_rest = Some(item.to_string());
                        }
                    } else {
                        left_empty = true;
                    }
                }
                if i % 2 == 1 {
                    if let Some(item) = right_iter.next() {
                        let mut item: String = item.to_string();
                        if let Some(rest) = right_rest {
                            item = rest + &item;
                            right_rest = None;
                        }

                        if !item.is_empty() && item.chars().last().unwrap() == '\n' {
                            result.push(item.to_string());
                        } else if !item.is_empty() {
                            right_rest = Some(item.to_string());
                        }
                    } else {
                        right_empty = true;
                    }
                }
                if left_empty && right_empty {
                    break;
                } else {
                    i += 1;
                }
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

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "wrong number of arguments",
        ));
    }

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app: App = App::new(Path::new(&args[1]))?;

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

fn whole_mem() -> Vec<String> {
    let left_str = read_file_to_string(&"testdata\\left.log".to_string());
    let right_str = read_file_to_string(&"testdata\\right.log".to_string());

    let mut left_iter = left_str.split_inclusive('\n');
    let mut right_iter = right_str.split_inclusive('\n');

    let mut result: Vec<String> = Vec::new();

    let mut i: usize = 0;
    let mut left_empty: bool = false;
    let mut right_empty: bool = false;
    loop {
        if i % 2 == 0 {
            if let Some(item) = left_iter.next() {
                if !item.is_empty() && item.chars().last().unwrap() == '\n' {
                    result.push(item.to_string());
                }
            } else {
                left_empty = true;
            }
        }
        if i % 2 == 1 {
            if let Some(item) = right_iter.next() {
                if !item.is_empty() && item.chars().last().unwrap() == '\n' {
                    result.push(item.to_string());
                }
            } else {
                right_empty = true;
            }
        }
        if left_empty && right_empty {
            return result;
        } else {
            i += 1;
        }
    }
}

fn test(r: &Vec<String>) {
    let rr: Vec<String> = r
        .iter()
        .filter_map(|it| {
            if it != "Left\n" && it != "Right\n" {
                Some(it.clone())
            } else {
                None
            }
        })
        .collect();
    println!("{:?}", rr);
}

fn text_as_bytes() {
    let text: String = "Hello Darkness\nMy old friend.".to_string();
    let b = text.as_bytes();
    println!("{:?}", b);

    for it in b.split_inclusive(|c| *c == '\n' as u8) {
        println!("{:?}", it);
    }

    let mut tb: Vec<u8> = Vec::new();
    tb.append(&mut b.iter().cloned().collect());
}

fn get_line_breaks(text_str: &String) -> Vec<usize> {
    let mut line_breaks: Vec<usize> = Vec::new();
    line_breaks.push(0);
    let mut find_text = &text_str[0..];
    while let Some(next) = find_text.find('\n') {
        match line_breaks.last() {
            None => line_breaks.push(next + 1),
            Some(l) => line_breaks.push(l + next + 1),
        };
        find_text = &find_text[next + 1..];
    }
    line_breaks.push(text_str.len() - 1);
    line_breaks
}

fn get_ith_line<'a>(text: &'a String, i: usize, line_breaks: &Vec<usize>) -> &'a str {
    if line_breaks.len() < i + 1 {
        panic!("too much");
    }

    let res = &text[line_breaks[i]..line_breaks[i + 1]];
    if res.chars().last() == Some('\n') {
        return &text[line_breaks[i]..line_breaks[i + 1] - 1];
    } else {
        res
    }
}

fn main() {
    // block_on(run());
    // block_on(file_run());

    // println!("by blocks:");
    // let start = Instant::now();
    // let r = read_and_merge();
    // let duration = start.elapsed();
    // println!("{}, {:?}", r.len(), duration);

    // println!("\nall at once:");
    // let start = Instant::now();
    // let r = whole_mem();
    // let duration = start.elapsed();
    // println!("{}, {:?}", r.len(), duration);

    let text: String = String::from(
        "We did the slice.\nIt was the spooky slice.\nNow our swings have some spice.\nSpoooooky.\nSlice.",
    );

    let line_breaks: Vec<usize> = get_line_breaks(&text);
    println!("{:?}", line_breaks);

    for i in 0..line_breaks.len() - 1 {
        println!(">{}<", get_ith_line(&text, i, &line_breaks));
    }

    let left_str = read_file_to_string("testdata\\left.log".to_string());
    println!("load done.");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();

    let big_lbs: Vec<usize> = get_line_breaks(&left_str);
    println!("line breaks calculated.");
    println!(">{}<", get_ith_line(&left_str, 1000000, &big_lbs));

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
}
