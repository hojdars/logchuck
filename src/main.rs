use std::{env, io};
use tui::{backend::CrosstermBackend, Terminal};

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

    let text = vec![tui::text::Spans::from(args[1].clone())];
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = tui::widgets::Block::default()
                .title(" Found logfiles ")
                .borders(tui::widgets::Borders::ALL);
            let paragraph = tui::widgets::Paragraph::new(text.clone())
                .style(
                    tui::style::Style::default()
                        .bg(tui::style::Color::White)
                        .fg(tui::style::Color::Black),
                )
                .block(block)
                .alignment(tui::layout::Alignment::Left);
            f.render_widget(paragraph, size);
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
