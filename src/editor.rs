use crate::{table::Cell, Document, Terminal};
use std::env;
use std::time::{Duration, Instant};
use termion::{color, event::Key};

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_MESSAGE_DURATION: Duration = Duration::from_secs(5);
const COLOR_FORMAT_LENGTH: usize = 45;

#[derive(Default, PartialEq, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() - self.time >= STATUS_MESSAGE_DURATION
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cell_index: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    copy: Vec<Cell>,
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                Terminal::cursor_show();
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status =
            String::from("HELP: Ctrl-q to Quit, Ctrl-s to Save, Return to Edit");

        let document = if let Some(file_name) = args.get(1) {
            if !file_name.ends_with(".csv") {
                initial_status = String::from(
                    "Warning: This editor currently only supports utf-8 encoded csv files.",
                );
            }
            match Document::open(file_name) {
                Ok(doc) => doc,
                Err(_) => {
                    initial_status = String::from("Err: Couldn't open file");
                    Document::default()
                }
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to init terminal"),
            document,
            cell_index: Position { x: 1, y: 2 },
            offset: Position { x: 0, y: 1 },
            status_message: StatusMessage::from(initial_status),
            copy: Vec::new(),
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
        } else {
            self.draw_table();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cell_index.x.saturating_sub(self.offset.x),
                y: self.cell_index.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::flush()
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.set_status("Not Saving");
                return;
            }
            self.document.file_name = new_name;
        }

        match self.document.save() {
            Ok(_) => self.set_status("Saved!"),
            Err(_) => self.set_status("Error: Unable to save changes"),
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.handle_quit()?,
            Key::Ctrl('s') => self.save(),
            Key::Char(c) => {
                match c {
                    '\n' => self.handle_insert(pressed_key)?,
                    '=' => self.handle_statistics(),
                    _ => {}
                }
                return Ok(());
            }
            Key::Ctrl('c') => self.handle_copy(),
            Key::Ctrl('v') => self.handle_paste(pressed_key)?,
            Key::Ctrl('x') => self.handle_cut(pressed_key),
            Key::Delete => self.handle_delete(pressed_key),
            Key::Ctrl('z') => self.handle_undo(pressed_key)?,
            Key::CtrlLeft | Key::CtrlRight | Key::CtrlUp | Key::CtrlDown => {
                self.handle_highlight_selection(pressed_key)?;
                return Ok(());
            }
            Key::ShiftUp => {
                self.document.highlight(&self.cell_index);
                self.highlight_row(1, self.cell_index.y);
                return Ok(());
            }
            Key::ShiftDown => {
                self.document.highlight(&self.cell_index);
                self.highlight_row(self.cell_index.y, self.document.table.num_rows() + 1);
                return Ok(());
            }
            Key::ShiftLeft => {
                self.document.highlight(&self.cell_index);
                self.highlight_col(1, self.cell_index.x);
                return Ok(());
            }
            Key::ShiftRight => {
                self.document.highlight(&self.cell_index);
                self.highlight_col(self.cell_index.x, self.document.table.num_cols() + 1);
                return Ok(());
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => {
                self.move_position(pressed_key);
            }
            _ => (),
        }

        self.update_document_dimensions();
        self.handle_boundary_conditions()?;
        self.document.highlight(&self.cell_index);
        self.scroll();
        Ok(())
    }

    fn highlight_row(&mut self, starty: usize, endy: usize) {
        if starty < 1 && endy > self.document.table.num_rows() {
            return;
        }
        for y in starty..endy {
            let pos = Position {
                x: self.cell_index.x,
                y,
            };
            self.document.multi_highlight(&pos);
        }
    }

    fn highlight_col(&mut self, startx: usize, endx: usize) {
        if startx < 1 || endx > self.document.table.num_cols() + 1 {
            return;
        }
        for x in startx..endx {
            let pos = Position {
                x,
                y: self.cell_index.y,
            };
            self.document.multi_highlight(&pos);
        }
    }
    fn scroll(&mut self) {
        let Position { x, y } = self.cell_index;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize - 1;

        self.scroll_vertical(y, height);
        self.scroll_horizontal(x, width);
    }

    fn move_position(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let height = self.document.table.num_rows();
        let width = self.document.table.num_cols();
        let Position { mut x, mut y } = self.cell_index;

        match key {
            Key::Up => y = y.saturating_sub(1).max(0),
            Key::Down if y <= height => y = y.saturating_add(1),
            Key::Left => x = x.saturating_sub(1).max(0),
            Key::Right if x <= width => x = x.saturating_add(1),
            Key::PageUp => {
                y = if y > terminal_height + 1 {
                    y.saturating_sub(terminal_height)
                } else {
                    1
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height - 2
                }
            }
            Key::Home => x = 1,
            Key::End => x = width,
            _ => {}
        }

        self.cell_index = Position { x, y };
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("CLICSV -- version: {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!(
            "{}{}{}",
            (self.terminal.size().height / 3).to_string(),
            spaces,
            welcome_message
        );
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn draw_status_bar(&self) {
        let width = self.terminal.size().width as usize;
        let modified_indicator = if !self.document.is_saved() {
            " (modified)"
        } else {
            ""
        };

        let mut file_name = self
            .document
            .file_name
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| "[No Name]".to_string());
        file_name.truncate(20);

        let mut status = format!(
            "{} - rows:{} cols:{}{}",
            file_name,
            self.document.table.num_rows(),
            self.document.table.num_cols(),
            modified_indicator
        );

        let line_indicator = format!(
            "y: {}/{} x: {}/{}",
            self.cell_index.y,
            self.document.table.num_rows(),
            self.cell_index.x,
            self.document.table.num_cols()
        );

        #[allow(clippy::integer_arithmetic)]
        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        if !self.status_message.is_expired() {
            let mut text = self.status_message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn draw_row(&self, ridx: u16) {
        let ncols = self.document.table.num_cols();
        let width = self.terminal.size().width as usize;
        let row = self.document.get_row((ridx as usize) + self.offset.y - 1);
        let nrows = self.document.table.num_rows();

        if row.len() != ncols {
            Terminal::clear_screen();
            println!("Error: rows have unequal amount of columns. Exiting...");
            std::process::exit(1);
        }

        let mut row_str = String::new();
        let mut diff = 0;

        for i in self.offset.x..ncols {
            let cell = &row[i];
            let filling_width = self.document.table.column_width(cell.x_loc) - cell.width;

            let s = if cell.highlighted {
                diff += COLOR_FORMAT_LENGTH;
                format!(
                    "{}{}{}{}{}{} {} ",
                    color::Fg(STATUS_FG_COLOR),
                    color::Bg(STATUS_BG_COLOR),
                    cell.contents,
                    " ".repeat(filling_width),
                    color::Bg(color::Reset),
                    color::Fg(color::Reset),
                    "│"
                )
            } else {
                format!("{}{} {} ", cell.contents, " ".repeat(filling_width), "│")
            };

            row_str.push_str(&s);
            if row_str.len() > width + diff {
                break;
            }
        }

        let len_term_str = (ridx as usize) + self.offset.y - 2;
        let row_filling = nrows.to_string().len() - len_term_str.to_string().len();
        let terminal_row_str = format!("{}{}", len_term_str, " ".repeat(row_filling));

        println!(
            "{}{}│{}{}\r",
            color::Fg(STATUS_FG_COLOR),
            terminal_row_str,
            color::Fg(color::Reset),
            row_str
        );
    }

    fn draw_header(&self) {
        let width = self.terminal.size().width as usize;
        let ncols = self.document.table.num_cols();
        let nrows = self.document.table.num_rows();

        let mut col_str = String::new();
        for x in (self.offset.x + 1)..(ncols + 1) {
            let fill = self.document.table.column_width(x) - 1;
            col_str.push_str(&format!("{}{} {} ", num_to_let(x), " ".repeat(fill), "|"));
        }

        let row_fill = nrows.to_string().len() + 1;
        col_str = format!(
            "{}{}{}",
            color::Fg(STATUS_FG_COLOR),
            " ".repeat(row_fill),
            col_str
        );
        col_str.truncate(width);

        println!("{}\r", col_str);
        Terminal::clear_current_line();
        println!("{}\r", "-".repeat(width));
    }

    fn draw_table(&self) {
        let height = self.terminal.size().height;
        let nrows = self.document.table.num_rows();

        Terminal::clear_current_line();
        self.draw_header();

        for terminal_row in 2..height {
            Terminal::clear_current_line();

            if terminal_row as usize <= nrows + 1 && !self.document.is_empty() {
                self.draw_row(terminal_row - 1);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                let edgenumber = terminal_row - 2;
                println!("{}{}\r", color::Fg(STATUS_FG_COLOR), edgenumber);
            }
        }
    }

    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();

        loop {
            self.set_status(&format!("{}{}", prompt, result));
            self.refresh_screen()?;

            match Terminal::read_key()? {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) if !c.is_control() => result.push(c),
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
        }

        self.status_message = StatusMessage::from(String::new());
        Ok(if result.is_empty() {
            None
        } else {
            Some(result)
        })
    }

    // Helper methods
    fn set_status(&mut self, message: &str) {
        self.status_message = StatusMessage::from(message.to_string());
    }

    fn handle_quit(&mut self) -> Result<(), std::io::Error> {
        if !self.document.is_saved() {
            self.set_status("WARNING! File has unsaved changes. Press Ctrl-Q to quit");
            self.refresh_screen()?;
            if Terminal::read_key()? == Key::Ctrl('q') {
                self.should_quit = true;
            }
        } else {
            self.should_quit = true;
        }
        Ok(())
    }

    fn handle_insert(&mut self, key: Key) -> Result<(), std::io::Error> {
        if let Some(content) = self.prompt("INSERT: ")? {
            self.document.last_action.cells_affected = self.document.get_highlight_cells();
            self.document.last_action.key = key;
            let mut ins_string = content;
            ins_string.push(' ');
            self.document.insert(&self.cell_index, &ins_string);
        } else {
            self.set_status("Not Saved");
        }
        Ok(())
    }

    fn handle_statistics(&mut self) {
        match self.document.table.calc_summary() {
            Err(e) => self.set_status(&e),
            Ok((n, sum, mean, std)) => {
                self.set_status(&format!(
                    "Statistics for selected cells: n = {}, sum = {}, mean = {}, std = {}",
                    n, sum as f32, mean as f32, std as f32
                ));
            }
        }
    }

    fn handle_copy(&mut self) {
        self.copy = self.document.copy().unwrap_or_default();
        self.set_status("Copied");
    }

    fn handle_paste(&mut self, key: Key) -> Result<(), std::io::Error> {
        if self.copy.is_empty() {
            self.set_status("Error: Nothing to paste");
            return Ok(());
        }
        self.document.last_action.key = key;
        self.document.paste(&self.cell_index, &self.copy.clone())?;
        self.set_status("Pasted");
        Ok(())
    }

    fn handle_cut(&mut self, key: Key) {
        self.document.last_action.cells_affected = self.document.get_highlight_cells();
        self.document.last_action.key = key;
        self.copy = self.document.copy().unwrap_or_default();
        self.document.delete();
        self.set_status("Cut");
    }

    fn handle_delete(&mut self, key: Key) {
        self.document.last_action.key = key;
        self.document.last_action.cells_affected = self.document.get_highlight_cells();
        self.document.delete();
        self.set_status("Deleted.");
    }

    fn handle_undo(&mut self, key: Key) -> Result<(), std::io::Error> {
        self.document.undo();
        if self.document.last_action.key == key {
            self.set_status("Cannot undo more than one event.");
        } else {
            self.set_status("Undone.");
        }
        Ok(())
    }

    fn handle_highlight_selection(&mut self, key: Key) -> Result<(), std::io::Error> {
        self.set_status("Selection mode.");
        let mut count = 1;
        let mut next_key = key;

        match key {
            Key::CtrlLeft => {
                while next_key == Key::CtrlLeft {
                    let startx = self.cell_index.x.saturating_sub(count);
                    self.highlight_col(startx, self.cell_index.x);
                    self.refresh_screen()?;
                    next_key = Terminal::read_key()?;
                    count += 1;
                }
            }
            Key::CtrlRight => {
                while next_key == Key::CtrlRight {
                    self.highlight_col(self.cell_index.x, self.cell_index.x + count);
                    self.refresh_screen()?;
                    next_key = Terminal::read_key()?;
                    count += 1;
                }
            }
            Key::CtrlUp => {
                while next_key == Key::CtrlUp {
                    let starty = self.cell_index.y.saturating_sub(count);
                    self.highlight_row(starty, self.cell_index.y);
                    self.refresh_screen()?;
                    next_key = Terminal::read_key()?;
                    count += 1;
                }
            }
            Key::CtrlDown => {
                while next_key == Key::CtrlDown {
                    self.highlight_row(self.cell_index.y, self.cell_index.y + count);
                    self.refresh_screen()?;
                    next_key = Terminal::read_key()?;
                    count += 1;
                }
            }
            _ => {}
        }

        self.set_status("Stopped selection.");
        Ok(())
    }

    fn update_document_dimensions(&mut self) {
        let num_rows = self.document.table.num_rows();
        let num_cols = self.document.table.num_cols();

        if self.cell_index.y > num_rows {
            self.document.insert_newrow(&self.cell_index);
        }
        if self.cell_index.x > num_cols {
            self.document.insert_newcol(&self.cell_index);
        }
    }

    fn handle_boundary_conditions(&mut self) -> Result<(), std::io::Error> {
        if self.cell_index.y == 0 {
            self.cell_index.y += 1;
            self.document.highlight(&self.cell_index);
            self.highlight_row(1, self.document.table.num_rows() + 1);
            return Ok(());
        }
        if self.cell_index.x == 0 {
            self.cell_index.x += 1;
            self.document.highlight(&self.cell_index);
            self.highlight_col(1, self.document.table.num_cols() + 1);
            return Ok(());
        }
        Ok(())
    }

    fn scroll_vertical(&mut self, y: usize, height: usize) {
        if y < self.offset.y {
            self.offset.y = y;
        } else if y >= self.offset.y.saturating_add(height) {
            self.offset.y = y.saturating_sub(height).saturating_add(1);
        }
    }

    fn scroll_horizontal(&mut self, x: usize, width: usize) {
        const COLUMN_SEPARATOR_WIDTH: usize = 4;
        let mut strlen = 0;

        for i in self.offset.x..(x + 1) {
            strlen += self.document.table.column_width(i);
            strlen += COLUMN_SEPARATOR_WIDTH;
        }

        if strlen.saturating_sub(COLUMN_SEPARATOR_WIDTH)
            <= self.document.table.column_width(self.offset.x)
            && self.offset.x >= 1
        {
            self.offset.x = self.offset.x.saturating_sub(1);
        } else if strlen >= self.offset.x.saturating_add(width) {
            self.offset.x = self.offset.x.saturating_add(1);
        }
    }
}
fn num_to_let(num: usize) -> char {
    const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    if num % 26 == 0 {
        return 'A';
    }

    let idx = if num > 26 {
        let div = (num / 26) * 26;
        num - div
    } else {
        num
    };

    ALPHABET.chars().nth(idx - 1).unwrap()
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}\n", e);
}
