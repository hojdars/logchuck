use std::{env, io, path::Path};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

fn scan_directory(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
    let mut result: Vec<String> = Vec::new();
    for item in std::fs::read_dir(path)? {
        match item?.file_name().into_string() {
            Ok(file) => if file.as_bytes()[0] != b'.' { result.push(file) },
            Err(err_file) => return Err(io::Error::new(io::ErrorKind::Other, format!("filename is not Unicode, filename={:?}", err_file))),
        }
    }

    result.sort();
    Ok(result)
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

    let text = scan_directory(Path::new(&args[1]))?;
    let text: Vec<Spans> = text
        .iter()
        .map(|t| Spans::from(t.as_str()))
        .collect();

    loop {
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let paragraph = Paragraph::new("\nLogfiles | Logs | Settings")
                .style(Style::default().bg(Color::White).fg(Color::Black))
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Center);
            f.render_widget(paragraph, chunks[0]);

            let paragraph = Paragraph::new(text.clone())
                .style(Style::default().bg(Color::White).fg(Color::Black))
                .block(
                    Block::default()
                        .title(" Found logfiles ")
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Left);
            f.render_widget(paragraph, chunks[1]);
        })?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if let crossterm::event::KeyCode::Char('q') = key.code {
                break;
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
