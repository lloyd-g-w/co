//! The terminal UI. Rendering, input and UI state only: everything else is
//! the server's, reached through `co-client`.

use co_client::Client;
use co_protocol::{ClientKind, Hello};
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

pub struct Config {
    pub server_url: String,
}

struct App {
    server_url: String,
    status: String,
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    let mut app = App {
        server_url: config.server_url,
        status: "connecting…".into(),
    };
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app).await;
    ratatui::restore();
    result
}

async fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    let hello = Hello {
        name: "tui".into(),
        client: ClientKind::Tui,
        tools: false,
        host_id: None,
        token: None,
    };
    let connect = Client::connect(&app.server_url, hello);
    tokio::pin!(connect);
    let mut connected = false;
    let mut client = None;
    let mut input = EventStream::new();
    loop {
        terminal.draw(|f| draw(f, app))?;
        tokio::select! {
            res = &mut connect, if !connected => {
                connected = true;
                match res {
                    Ok((c, _events)) => {
                        app.status = format!("connected (client {})", c.hello.client_id);
                        client = Some(c);
                    }
                    Err(e) => app.status = format!("not connected: {e}"),
                }
            }
            Some(Ok(event)) = input.next() => {
                if let Event::Key(key) = event
                    && key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                {
                    drop(client);
                    return Ok(());
                }
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &App) {
    let [main, status] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());
    frame.render_widget(
        Paragraph::new("Welcome to co.\n\nPress q to quit.").block(Block::bordered().title(" co ")),
        main,
    );
    frame.render_widget(
        Line::from(format!(" {} · {}", app.server_url, app.status)).dim(),
        status,
    );
}
