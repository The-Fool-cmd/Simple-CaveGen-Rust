use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

// Constants used for grid sizes
const GRID_W: u16 = 20;
const GRID_H: u16 = 12;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Default, Debug)]
pub struct App {
    size: (u16, u16),
    cursor_x: u16,
    cursor_y: u16,
    exit: bool,
}

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
            KeyCode::Right => self.cursor_x = (self.cursor_x + 1).min(GRID_W - 1),
            KeyCode::Up => self.cursor_y = self.cursor_y.saturating_sub(1),
            KeyCode::Down => self.cursor_y = (self.cursor_y + 1).min(GRID_H - 1),
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

        let counter_text = Text::from(vec![Line::from(vec![
            " Cursor Position: ".into(),
            self.cursor_x.to_string().yellow(),
            " ".into(),
            self.cursor_y.to_string().blue(),
        ])]);

        let paragraph = Paragraph::new(counter_text).centered().block(block);

        frame.render_widget(paragraph, area);
    }
}
