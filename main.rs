use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

// Constants used for grid sizes
const WORLD_W: usize = 160;
const WORLD_H: usize = 90;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug)]
pub struct App {
    size: (u16, u16),
    cursor_x: usize,
    cursor_y: usize,
    exit: bool,
    grid: Grid,
    cam_x: usize,
    cam_y: usize,
    view_w: usize,
    view_h: usize,
}
#[derive(Debug)]
struct Grid {
    w: usize,
    h: usize,
    cells: Vec<bool>,
}

impl Grid {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            cells: vec![false; w * h],
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.w && y < self.h
    }

    fn get(&self, x: usize, y: usize) -> Option<bool> {
        self.in_bounds(x, y).then(|| self.cells[self.idx(x, y)])
    }

    fn toggle(&mut self, x: usize, y: usize) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            self.cells[i] = !self.cells[i];
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            if event::poll(Duration::from_millis(50))? {
                self.handle_events()?;
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        self.ui(frame);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event.code)
            }
            Event::Resize(width, height) => self.size = (width, height),
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.cursor_x = self.cursor_x.saturating_sub(1),
            KeyCode::Right => self.cursor_x = (self.cursor_x + 1).min(self.grid.w - 1),
            KeyCode::Up => self.cursor_y = self.cursor_y.saturating_sub(1),
            KeyCode::Down => self.cursor_y = (self.cursor_y + 1).min(self.grid.h - 1),
            KeyCode::Char(' ') => self.grid.toggle(self.cursor_x, self.cursor_y),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn ui(&self, frame: &mut Frame) {
        let area = frame.area();

        let title = Line::from(" Cave! ".bold());
        let instructions = Line::from(vec![
            " Move cursor using arrow keys  ".into(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let debug_text = Text::from(vec![Line::from(vec![
            " Cursor Position: ".into(),
            self.cursor_x.to_string().yellow(),
            " ".into(),
            self.cursor_y.to_string().blue(),
        ])]);

        frame.render_widget(Paragraph::new(debug_text), chunks[0]);

        let inner_w_chars = chunks[1].width.saturating_sub(2);
        let inner_h_rows = chunks[1].height.saturating_sub(2);

        let view_w = (inner_w_chars as usize).max(1);
        let view_h = ((inner_h_rows as usize) / 2).max(1);

        let start_x = self.cam_x;
        let start_y = self.cam_y;

        let end_x = (start_x + view_w).min(self.grid.w);
        let end_y = (start_y + view_h).min(self.grid.h);

        // Grid
        let mut rows: Vec<Line> = Vec::with_capacity(end_y - start_y);
        for y in start_y..end_y {
            let mut spans: Vec<Span> = Vec::with_capacity(end_x - start_x);

            for x in start_x..end_x {
                let filled = self.grid.get(x, y).unwrap_or(false);
                let cell = if filled { "██" } else { "  " };

                let span = if x == self.cursor_x && y == self.cursor_y {
                    Span::from(cell).reversed()
                } else {
                    Span::from(cell)
                };

                spans.push(span);
            }
            rows.push(Line::from(spans));
        }
        let grid_text = Text::from(rows);
        let grid_paragraph = Paragraph::new(grid_text).block(block);

        frame.render_widget(grid_paragraph, chunks[1]);
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            size: (0, 0),
            cursor_x: 0,
            cursor_y: 0,
            exit: false,
            grid: Grid::new(WORLD_W, WORLD_H),
            cam_x: 0,
            cam_y: 0,
            view_w: 0,
            view_h: 0,
        }
    }
}
