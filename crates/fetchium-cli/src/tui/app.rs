//! TUI application state and render loop (PRD §42).

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::io::IsTerminal;

#[derive(Debug, PartialEq)]
enum Panel {
    Search,
    Results,
    Preview,
}

struct App {
    search_input: String,
    results: Vec<(String, String, String)>, // (title, url, snippet)
    selected: usize,
    preview: String,
    active_panel: Panel,
    status: String,
}

impl App {
    fn new() -> Self {
        Self {
            search_input: String::new(),
            results: vec![],
            selected: 0,
            preview: String::new(),
            active_panel: Panel::Search,
            status: " Tab: switch panel | Enter: search/preview | \u{2191}\u{2193}: navigate | Ctrl+C: quit".into(),
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            return false;
        }
        match self.active_panel {
            Panel::Search => match code {
                KeyCode::Char(c) => self.search_input.push(c),
                KeyCode::Backspace => {
                    self.search_input.pop();
                }
                KeyCode::Enter => {
                    if !self.search_input.is_empty() {
                        self.results = vec![(
                            format!("Result for '{}'", self.search_input),
                            "https://example.com".into(),
                            "Placeholder result. Connect to search engine for live results.".into(),
                        )];
                        self.active_panel = Panel::Results;
                        self.status = format!(
                            " {} result(s) | Tab: switch | \u{2191}\u{2193}: navigate",
                            self.results.len()
                        );
                    }
                }
                KeyCode::Tab => self.active_panel = Panel::Results,
                _ => {}
            },
            Panel::Results => match code {
                KeyCode::Up => {
                    self.selected = self.selected.saturating_sub(1);
                    self.update_preview();
                }
                KeyCode::Down => {
                    if self.selected + 1 < self.results.len() {
                        self.selected += 1;
                        self.update_preview();
                    }
                }
                KeyCode::Enter => {
                    self.update_preview();
                    self.active_panel = Panel::Preview;
                }
                KeyCode::Tab => self.active_panel = Panel::Search,
                KeyCode::Char('q') => return false,
                _ => {}
            },
            Panel::Preview => match code {
                KeyCode::Tab | KeyCode::Esc => self.active_panel = Panel::Results,
                KeyCode::Char('q') => return false,
                _ => {}
            },
        }
        true
    }

    fn update_preview(&mut self) {
        if let Some((title, url, snippet)) = self.results.get(self.selected) {
            self.preview = format!("# {title}\n{url}\n\n{snippet}");
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(1),
            ])
            .split(area);

        // Search bar
        let search_block = Block::default()
            .borders(Borders::ALL)
            .title("Search")
            .border_style(if self.active_panel == Panel::Search {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });
        let search = Paragraph::new(self.search_input.as_str()).block(search_block);
        frame.render_widget(search, chunks[0]);

        // Main split
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(chunks[1]);

        // Results list
        let items: Vec<ListItem> = self
            .results
            .iter()
            .enumerate()
            .map(|(i, (title, url, _))| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(vec![
                    Line::styled(format!("[{}] {}", i + 1, title), style),
                    Line::styled(format!("    {url}"), Style::default().fg(Color::Blue)),
                ])
            })
            .collect();
        let results_block = Block::default()
            .borders(Borders::ALL)
            .title("Results")
            .border_style(if self.active_panel == Panel::Results {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });
        let list = List::new(items).block(results_block);
        frame.render_widget(list, main_chunks[0]);

        // Preview panel
        let preview_text = if self.preview.is_empty() {
            "Select a result and press Enter to preview.".to_string()
        } else {
            self.preview.clone()
        };
        let preview_block = Block::default()
            .borders(Borders::ALL)
            .title("Preview")
            .border_style(if self.active_panel == Panel::Preview {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });
        let preview = Paragraph::new(preview_text)
            .wrap(Wrap { trim: true })
            .block(preview_block);
        frame.render_widget(preview, main_chunks[1]);

        // Status bar
        let status =
            Paragraph::new(self.status.as_str()).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[2]);
    }
}

/// Launch the interactive TUI.
pub fn run_tui() -> anyhow::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        anyhow::bail!(
            "TUI requires an interactive terminal (TTY). Run `fetchium tui` directly in a terminal session."
        );
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;
        if let Event::Key(key) = event::read()? {
            if !app.handle_key(key.code, key.modifiers) {
                break;
            }
        }
    }
    Ok(())
}
