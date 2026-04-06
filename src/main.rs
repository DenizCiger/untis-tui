use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, KeyEventKind,
    MouseEvent,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;
use untis_tui::app::state::{
    AbsenceChunkPayload, AppCommand, AppState, WorkerEvent, build_absence_chunk_request,
    build_bootstrap_payload, update_absence_chunk_progress,
};
use untis_tui::ui;
use untis_tui::webuntis::WebUntisClient;

#[derive(Debug)]
enum RuntimeEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Worker(WorkerEvent),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<RuntimeEvent>();
    spawn_input_thread(tx.clone());

    let demo_mode = std::env::args().skip(1).any(|arg| arg == "--demo");
    let mut state = if demo_mode {
        AppState::new_demo()
    } else {
        AppState::new()
    };
    if let Ok((width, height)) = crossterm::terminal::size() {
        state.update_terminal_size(width, height);
    }
    for command in state.initial_commands() {
        execute_command(tx.clone(), command);
    }

    let mut tick = tokio::time::interval(Duration::from_millis(250));
    loop {
        terminal.draw(|frame| ui::render(frame, &state))?;
        tokio::select! {
            Some(event) = rx.recv() => {
                match event {
                    RuntimeEvent::Key(key) => {
                        let commands = state.handle_key(key);
                        if handle_commands(&tx, &mut state, commands) {
                            break;
                        }
                    }
                    RuntimeEvent::Mouse(mouse) => {
                        let commands = state.handle_mouse(mouse);
                        if handle_commands(&tx, &mut state, commands) {
                            break;
                        }
                    }
                    RuntimeEvent::Resize(width, height) => state.update_terminal_size(width, height),
                    RuntimeEvent::Worker(event) => {
                        let commands = state.handle_worker_event(event);
                        if handle_commands(&tx, &mut state, commands) {
                            break;
                        }
                    }
                }
            }
            _ = tick.tick() => {}
        }
    }

    Ok(())
}

fn handle_commands(
    tx: &mpsc::UnboundedSender<RuntimeEvent>,
    _state: &mut AppState,
    commands: Vec<AppCommand>,
) -> bool {
    let mut should_quit = false;
    for command in commands {
        match command {
            AppCommand::Quit => should_quit = true,
            command => execute_command(tx.clone(), command),
        }
    }
    should_quit
}

fn spawn_input_thread(tx: mpsc::UnboundedSender<RuntimeEvent>) {
    std::thread::spawn(move || {
        loop {
            if !event::poll(Duration::from_millis(100)).unwrap_or(false) {
                continue;
            }
            match event::read() {
                Ok(CrosstermEvent::Key(key)) => {
                    if key.kind == KeyEventKind::Press {
                        let _ = tx.send(RuntimeEvent::Key(key));
                    }
                }
                Ok(CrosstermEvent::Mouse(mouse)) => {
                    let _ = tx.send(RuntimeEvent::Mouse(mouse));
                }
                Ok(CrosstermEvent::Resize(width, height)) => {
                    let _ = tx.send(RuntimeEvent::Resize(width, height));
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
}

fn execute_command(tx: mpsc::UnboundedSender<RuntimeEvent>, command: AppCommand) {
    match command {
        AppCommand::Bootstrap => {
            tokio::spawn(async move {
                let payload = build_bootstrap_payload();
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::BootstrapLoaded(payload)));
            });
        }
        AppCommand::ValidateLogin(config) => {
            tokio::spawn(async move {
                let result = WebUntisClient::test_credentials(&config)
                    .await
                    .map(|_| config)
                    .map_err(|error| error.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::LoginValidated(result)));
            });
        }
        AppCommand::LoadTimetableNetwork {
            request_id,
            config,
            week_date,
            target,
        } => {
            tokio::spawn(async move {
                let result = WebUntisClient::fetch_week_timetable(&config, week_date, &target)
                    .await
                    .map_err(|error| error.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::TimetableLoaded {
                    request_id,
                    week_date,
                    target,
                    result,
                }));
            });
        }
        AppCommand::LoadSearchIndex {
            profile_key,
            config,
        } => {
            tokio::spawn(async move {
                let result = WebUntisClient::fetch_timetable_search_index(&config)
                    .await
                    .map_err(|error| error.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::SearchIndexLoaded {
                    profile_key,
                    result,
                }));
            });
        }
        AppCommand::LoadAbsenceChunk {
            generation,
            config,
            base_date,
            chunk_index,
            is_initial,
        } => {
            tokio::spawn(async move {
                let result = load_absence_chunk(&config, base_date, chunk_index, is_initial)
                    .await
                    .map_err(|error| error.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::AbsencesLoaded {
                    generation,
                    is_initial,
                    result,
                }));
            });
        }
        AppCommand::Quit => {}
    }
}

async fn load_absence_chunk(
    config: &untis_tui::models::Config,
    base_date: chrono::NaiveDate,
    chunk_index: usize,
    is_initial: bool,
) -> Result<AbsenceChunkPayload, untis_tui::webuntis::WebUntisError> {
    let mut all_items = Vec::new();
    let mut next_chunk_index = chunk_index;
    let mut empty_chunk_streak = 0;
    let mut has_more = true;
    let mut days_loaded = chunk_index * 45;

    for (index, (range_start, range_end)) in
        build_absence_chunk_request(base_date, chunk_index, is_initial)
            .into_iter()
            .enumerate()
    {
        let items =
            WebUntisClient::fetch_absences_for_range(config, range_start, range_end).await?;
        let (updated_chunk_index, updated_empty_streak, updated_has_more, updated_days_loaded) =
            update_absence_chunk_progress(chunk_index + index, empty_chunk_streak, items.len());
        next_chunk_index = updated_chunk_index;
        empty_chunk_streak = updated_empty_streak;
        has_more = updated_has_more;
        days_loaded = updated_days_loaded;
        all_items.extend(items);

        if is_initial || all_items.len() >= 12 || !has_more {
            break;
        }
    }

    Ok(AbsenceChunkPayload {
        items: all_items,
        next_chunk_index,
        empty_chunk_streak,
        has_more,
        days_loaded,
    })
}
