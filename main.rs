use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::widgets::Wrap;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

// Constants used for grid sizes
const WORLD_W: usize = 80;
const WORLD_H: usize = 42;
// Tick duration in ms
const TICK: Duration = Duration::from_millis(50);
// Constants used for the layout
const DEBUG_COLS: u16 = 30;
// Constant used for DrunkWalk Gen
const DRUNKCHANCE: f64 = 0.4;
// Constant used for Random Gen
const RANDCHANCE: f64 = 0.45;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
#[derive(Debug)]
enum Algorithm {
    Paint,
    Life,
    DrunkWalk,
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
    seed: u64,
    algo: Algorithm,
    running: bool,
    last_tick: Instant,
}
#[derive(Debug)]
struct Grid {
    w: usize,
    h: usize,
    cells: Vec<bool>,
    next: Vec<bool>,
}

impl Grid {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            cells: vec![false; w * h],
            next: vec![false; w * h],
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

    fn set(&mut self, x: usize, y: usize, v: bool) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            self.cells[i] = v;
        }
    }

    fn fill(&mut self, v: bool) {
        self.cells.fill(v);
    }

    fn clear(&mut self) {
        self.fill(false);
    }

    fn step_life(&mut self) {
        for y in 0..self.h {
            for x in 0..self.w {
                let i = self.idx(x, y);
                let alive = self.cells[i];

                let mut n: u8 = 0;

                for dy in -1isize..=1 {
                    for dx in -1isize..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = x as isize + dx;
                        let ny = y as isize + dy;

                        if nx < 0 || ny < 0 {
                            continue;
                        }

                        let nx = nx as usize;
                        let ny = ny as usize;

                        if nx >= self.w || ny >= self.h {
                            continue;
                        }

                        if self.cells[self.idx(nx, ny)] {
                            n += 1;
                        }
                    }
                }

                self.next[i] = if alive { n == 2 || n == 3 } else { n == 3 };
            }
        }

        std::mem::swap(&mut self.cells, &mut self.next);
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let (w, h) = terminal::size()?;
        self.size = (w, h);
        while !self.exit {
            if event::poll(Duration::from_millis(50))? {
                self.handle_events()?;
            }
            if self.running && self.last_tick.elapsed() >= TICK {
                self.step_active();
                self.last_tick = Instant::now();
            }
            terminal.draw(|frame| self.ui(frame))?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event.code)
            }
            Event::Resize(width, height) => {
                self.size = (width, height);
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, code: KeyCode) {
        let mut moved = false;

        match code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Left => {
                self.cursor_x = self.cursor_x.saturating_sub(1);
                moved = true
            }
            KeyCode::Right => {
                self.cursor_x = (self.cursor_x + 1).min(self.grid.w - 1);
                moved = true
            }
            KeyCode::Up => {
                self.cursor_y = self.cursor_y.saturating_sub(1);
                moved = true
            }
            KeyCode::Down => {
                self.cursor_y = (self.cursor_y + 1).min(self.grid.h - 1);
                moved = true
            }
            KeyCode::Char(' ') => self.grid.toggle(self.cursor_x, self.cursor_y),
            KeyCode::Char('c') => self.grid.clear(),
            KeyCode::Char('r') => self.regen_random(RANDCHANCE),
            KeyCode::Char('n') => {
                self.seed += 1;
                self.regen_random(RANDCHANCE);
            }
            KeyCode::Char('p') => {
                self.running = !self.running;
                self.last_tick = Instant::now();
            }
            KeyCode::Char('s') => {
                self.step_active();
            }
            KeyCode::Char('1') => self.algo = Algorithm::Paint,
            KeyCode::Char('2') => {
                self.algo = Algorithm::Life;
                self.running = false;
                self.last_tick = Instant::now();
            }
            KeyCode::Char('3') => {
                self.algo = Algorithm::DrunkWalk;
                self.running = false;
                self.last_tick = Instant::now();
            }

            _ => {}
        }
        if moved {
            self.follow_cursor();
        }
    }

    fn ui(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let title = Line::from(" Cave! ".bold());
        let instructions = Line::from(vec![
            " Paint ".into(),
            "<1>".into(),
            " Life ".into(),
            "<2>".into(),
            " DrunkWalk ".into(),
            "<3>".into(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(DEBUG_COLS), Constraint::Min(0)])
            .split(area);

        let grid_area = chunks[1];
        let inner = block.inner(grid_area);

        self.view_w = ((inner.width as usize) / 2).max(1).min(self.grid.w);
        self.view_h = (inner.height as usize).max(1).min(self.grid.h);

        self.follow_cursor();

        let debug_text = Text::from(vec![Line::from(vec![
            " Cursor Position ".red().into(),
            format!("x: {} y: {}", self.cursor_x, self.cursor_y).into(),
            " Seed ".red().into(),
            self.seed.to_string().into(),
            " Algo: ".red().into(),
            format!("{:?}", self.algo).into(),
            " Inner ".red().into(),
            format!("{}x{}", inner.width, inner.height).into(),
            " View ".red().into(),
            format!("{}x{}", self.view_w, self.view_h).into(),
            " World ".red().into(),
            format!("{}x{}", self.grid.w, self.grid.h).into(),
            " Running ".red().into(),
            self.running.to_string().into(),
        ])]);

        frame.render_widget(
            Paragraph::new(debug_text).wrap(Wrap { trim: true }),
            chunks[0],
        );

        let start_x = self.cam_x;
        let start_y = self.cam_y;
        let end_x = (start_x + self.view_w).min(self.grid.w);
        let end_y = (start_y + self.view_h).min(self.grid.h);

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

    fn step_active(&mut self) {
        match self.algo {
            Algorithm::Paint => {}
            Algorithm::Life => {
                self.grid.step_life();
            }
            Algorithm::DrunkWalk => {
                // Increment seed to chance cave every step
                self.seed += 1;
                self.gen_drunk_walk(DRUNKCHANCE);
                // Move viewport to the center of the grid
                self.cursor_x = self.grid.w / 2 + self.view_w / 2;
                self.cursor_y = self.grid.h / 2 + self.view_h / 2;
                self.follow_cursor();
            }
        }
    }

    fn follow_cursor(&mut self) {
        // if viewport not initialized, leave
        if self.view_w == 0 || self.view_h == 0 {
            return;
        }
        // X: keep cursor within [cam_x, cam_x + view_w - 1]
        if self.cursor_x < self.cam_x {
            self.cam_x = self.cursor_x;
        } else if self.cursor_x >= self.cam_x + self.view_w {
            self.cam_x = self.cursor_x + 1 - self.view_w;
        }

        // Y
        if self.cursor_y < self.cam_y {
            self.cam_y = self.cursor_y;
        } else if self.cursor_y >= self.cam_y + self.view_h {
            self.cam_y = self.cursor_y + 1 - self.view_h;
        }

        // Clamp camera so viewport stays inside world
        if self.grid.w <= self.view_w {
            self.cam_x = 0;
        } else {
            self.cam_x = self.cam_x.min(self.grid.w - self.view_w);
        }

        if self.grid.h <= self.view_h {
            self.cam_y = 0;
        } else {
            self.cam_y = self.cam_y.min(self.grid.h - self.view_h);
        }
    }

    fn regen_random(&mut self, p: f64) {
        // clear grid before generation
        self.grid.clear();

        let mut rng = StdRng::seed_from_u64(self.seed);

        for y in 0..self.grid.h {
            for x in 0..self.grid.w {
                let border = x == 0 || y == 0 || x == self.grid.w - 1 || y == self.grid.h - 1;

                let wall = if border { true } else { rng.random_bool(p) };

                self.grid.set(x, y, wall);
            }
        }
    }
    fn gen_drunk_walk(&mut self, carve_target_ratio: f64) {
        // start as solid walls
        self.grid.fill(true);

        // seeded rng
        let mut rng = StdRng::seed_from_u64(self.seed);

        // start in the center
        let mut x = self.grid.w / 2;
        let mut y = self.grid.h / 2;

        // keep a 1-cell border as walls to avoid open edges
        x = x.clamp(1, self.grid.w.saturating_sub(2));
        y = y.clamp(1, self.grid.h.saturating_sub(2));

        // decide how much to carve
        let total = self.grid.w * self.grid.h;
        let target_open = ((total as f64) * carve_target_ratio).round() as usize;

        // carve until we hit target
        let mut opened = 0usize;

        // carve starting cell
        if self.grid.get(x, y) == Some(true) {
            self.grid.set(x, y, false);
            opened += 1;
        }

        while opened < target_open {
            // choose direction 0..4
            match rng.random_range(0..4) {
                0 => x = x.saturating_sub(1),
                1 => x = (x + 1).min(self.grid.w - 1),
                2 => y = y.saturating_sub(1),
                _ => y = (y + 1).min(self.grid.h - 1),
            }

            // enforce border walls
            x = x.clamp(1, self.grid.w.saturating_sub(2));
            y = y.clamp(1, self.grid.h.saturating_sub(2));

            // carve if still wall
            if self.grid.get(x, y) == Some(true) {
                self.grid.set(x, y, false);
                opened += 1;
            }
        }
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
            view_w: 1,
            view_h: 1,
            seed: 1,
            algo: Algorithm::Paint,
            last_tick: Instant::now(),
            running: false,
        }
    }
}
