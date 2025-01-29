use crate::Document;
use crate::Terminal;
use crate::table;

use std::env;
use std::time::{Duration, Instant};
use termion::{color, event::Key};
use table::Cell;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63,63,63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, PartialEq, Clone)]
pub struct Position 
{
    pub x: usize,
    pub y: usize,
}

struct StatusMessage 
{
    text: String,
    time: Instant,
}


impl StatusMessage{
    fn from(message: String) -> Self 
    {
        Self 
        {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor 
{
    should_quit: bool,
    terminal: Terminal,
    cell_index: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    copy: Vec<Cell>
}

impl Editor 
{
    pub fn run(&mut self) 
    {
        loop 
        {
            if let Err(error) = self.refresh_screen() 
            {
                die(error);
            }
            if self.should_quit 
            {
                Terminal::cursor_show();
                break;
            }
            if let Err(error) = self.process_keypress()
            {
                die(error);
            }
        }
    }

    pub fn default() -> Self 
    {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-q to Quit, Ctrl-s to Save, Return to Edit");
        let document = if let Some(file_name) = args.get(1) 
        {
            let doc = Document::open(file_name);
            if !file_name.ends_with(".csv")
            {
                initial_status = format!("Warning: This editor currently only supports utf-8 encoded csv files.");
            }
            if let Ok(doc) = doc 
            {
                doc
            }
            else 
            {
                initial_status = format!("Err: Couldn't open file");
                Document::default()
            }
        }
        else
        {
            Document::default()
        };

        Self 
        {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to init terminal"),
            document,
            cell_index: Position {x:1,y:2,},
            offset: Position {x:0,y:1},
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
    
    fn save(&mut self) 
    {
        if self.document.file_name.is_none() 
        {
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none()
            {
                self.status_message = StatusMessage::from(format!("Not Saving"));
                return;
            }
            self.document.file_name = new_name;
        }
        if self.document.save().is_ok()
        {
            self.status_message = StatusMessage::from(format!("Saved!"));
        }
        else 
        {
            self.status_message = StatusMessage::from(format!("Error: Unable to save changes"));
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => {
                if !self.document.is_saved(){
                    self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q to quit"
                    ));
                    self.refresh_screen()?;
                    let read = Terminal::read_key()?;
                    if read == Key::Ctrl('q'){
                        self.should_quit = true;
                    }
                    return Ok(());
                }
                else{
                    self.should_quit = true;
                }
            }
            //save file
            Key::Ctrl('s') => {
                self.save()
            },
            Key::Char(c) => {
                //enter data into cell at current position
                if c == '\n'{
                    let content = self.prompt("INSERT: ").unwrap_or(None);
                    if content.is_none(){
                        self.status_message = StatusMessage::from(format!("Not Saved"));
                    }
                    else
                    {
                        self.document.last_action.cells_affected = self.document.get_highlight_cells();
                        self.document.last_action.key = pressed_key;
                        let mut ins_string: String = content.unwrap();
                        ins_string.push(' ');
                        self.document.insert(&self.cell_index,&ins_string);
                    }
                }
                //get statstical infomation for highlighted cell
                if c == '='{
                    match self.document.table.calc_summary() {
                       Err(e) => {
                           self.status_message = StatusMessage::from(e);
                        },
                       Ok((n, sum, mean, std)) => {
                           self.status_message = StatusMessage::from(format!(
                               "Statitics for selected cells: n = {}, sum = {}, mean = {}, std = {}"
                               ,n, sum as f32, mean as f32, std as f32
                            ));
                        },
                    }
                }
                return Ok(());
            }
            //copy highlighted cell data
            Key::Ctrl('c') => {
                self.copy = self.document.copy().unwrap_or(Vec::new());
                self.status_message=StatusMessage::from(String::from("Copied"));
            }
            //paste copied data to current position
            Key::Ctrl('v') => {
                if self.copy.is_empty(){
                    self.status_message=StatusMessage::from(String::from("Error: Nothing to paste"));
                    return Ok(());
                } 
                self.document.last_action.key = pressed_key;
                self.document.paste(&self.cell_index,&self.copy.clone())?;
                self.status_message=StatusMessage::from(String::from("Pasted"));
            }
            //copy and delete highlighted cell data
            Key::Ctrl('x') => {
                self.document.last_action.cells_affected = self.document.get_highlight_cells();
                self.document.last_action.key = pressed_key;
                self.copy = self.document.copy().unwrap_or(Vec::new());
                self.document.delete();
                self.status_message=StatusMessage::from(String::from("Cut"));
            }
            //delete contents from highlighted cells
            Key::Delete =>{
                self.document.last_action.key = pressed_key;
                self.document.last_action.cells_affected = self.document.get_highlight_cells();
                self.document.delete();
                self.status_message=StatusMessage::from(String::from("Deleted."));
            }
            //undo the last edit to document
            Key::Ctrl('z') => {
                self.document.undo();
                if self.document.last_action.key == pressed_key{
                    self.status_message=StatusMessage::from(String::from("Cannot undo more than one event."));
                    return Ok(());
                }
                self.status_message=StatusMessage::from(String::from("Undone."));
            }
            //highlight cells to the given direction...
            Key::CtrlLeft => {
                self.status_message=StatusMessage::from(String::from("Selection mode."));
                let mut count :usize= 1;
                self.highlight_col(self.cell_index.x-count, self.cell_index.x);
                self.refresh_screen()?;
                let mut next_key = pressed_key;
                while next_key == Key::CtrlLeft{
                    count += 1;
                    let startx = self.cell_index.x.saturating_sub(count);
                    self.highlight_col(startx, self.cell_index.x);
                    self.refresh_screen()?;
                    next_key = Terminal::read_key()?;
                }
                self.status_message=StatusMessage::from(String::from("Stopped selection."));
                return Ok(());
            }
            Key::CtrlRight => {
                self.status_message=StatusMessage::from(String::from("Selection mode."));
                let mut count :usize= 1;
                self.highlight_col(self.cell_index.x, self.cell_index.x+count);
                self.refresh_screen()?;
                let mut next_key = pressed_key;
                while next_key == Key::CtrlRight{
                    count += 1;
                    self.highlight_col(self.cell_index.x, self.cell_index.x+count);
                    self.refresh_screen()?;
                    next_key = Terminal::read_key()?;
                }
                self.status_message=StatusMessage::from(String::from("Stopped selection."));
                return Ok(());
            }
            Key::CtrlUp => {
                self.status_message=StatusMessage::from(String::from("Selection mode."));
                let mut count :usize= 1;
                self.highlight_row(self.cell_index.y-count, self.cell_index.y);
                self.refresh_screen()?;
                let mut next_key = pressed_key;
                while next_key == Key::CtrlUp{
                    count += 1;
                    let starty = self.cell_index.y.saturating_sub(count);
                    self.highlight_row(starty, self.cell_index.y);
                    self.refresh_screen()?;
                    next_key = Terminal::read_key()?;
                }
                self.status_message=StatusMessage::from(String::from("Stopped selection."));
                return Ok(());
            }
            Key::CtrlDown => {
                self.status_message=StatusMessage::from(String::from("Selection mode."));
                let mut next_key: Key = pressed_key;
                let mut count :usize= 1;
                while next_key == Key::CtrlDown{
                    count += 1;
                    self.highlight_row(self.cell_index.y, self.cell_index.y+count);
                    self.refresh_screen()?;
                    next_key = Terminal::read_key()?;
                }
                self.status_message=StatusMessage::from(String::from("Stopped selection."));
                return Ok(());
            }
            //highlight all data from current positon to the end of document in the selected direction
            Key::ShiftUp => {
                self.document.highlight(&self.cell_index);
                self.highlight_row(1,self.cell_index.y);
                return Ok(());
            }
            Key::ShiftDown => {
                self.document.highlight(&self.cell_index);
                self.highlight_row(self.cell_index.y,self.document.table.num_rows()+1);
                return Ok(());
            }
            Key::ShiftLeft => {
                self.document.highlight(&self.cell_index);
                self.highlight_col(1,self.cell_index.x);
                return Ok(());
            }
            Key::ShiftRight => {
                self.document.highlight(&self.cell_index);
                self.highlight_col(self.cell_index.x,self.document.table.num_cols()+1);
                return Ok(());
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_position(pressed_key),
            _ => (),
        }

        //updating document information after actions
        let num_rows = self.document.table.num_rows();
        let num_cols = self.document.table.num_cols();        
        
        if self.cell_index.y > num_rows{
            self.document.insert_newrow(&self.cell_index);
        }
        if self.cell_index.x > num_cols{
            self.document.insert_newcol(&self.cell_index);
        }

        //if trying to escape the boundaries of a page, highlight the cells for that row/column
        if self.cell_index.y == 0{
            self.cell_index.y+=1;
            self.document.highlight(&self.cell_index);
            self.highlight_row(1,self.document.table.num_rows()+1);
            return Ok(());
        }
        if self.cell_index.x == 0{
            self.cell_index.x+=1;
            self.document.highlight(&self.cell_index);
            self.highlight_col(1,self.document.table.num_cols()+1);
            return Ok(());
        }
        self.document.highlight(&self.cell_index);
        self.scroll();
        Ok(())
    }

    //highight group of cells in the y direction
    fn highlight_row(&mut self,starty: usize, endy: usize){
        let mut pos: Position;
        let mut x: usize;
        if starty < 1 && endy > self.document.table.num_rows(){
            return;
        }
        for y in starty..endy{
            x = self.cell_index.x;
            pos = Position{x,y};
            self.document.multi_highlight(&pos);
        }
    }
    //highlight group of cells in the x direction
    fn highlight_col(&mut self, startx: usize, endx: usize){
        let mut pos: Position;
        let mut y: usize;
        if startx < 1 || endx > self.document.table.num_cols()+1{
            return;
        }
        for x in startx..endx{
            y = self.cell_index.y;
            pos= Position{x,y};
            self.document.multi_highlight(&pos);
        }
    }
    //changing current position and adjusting document crop to fit in terminal
    fn scroll(&mut self){
        let Position {x , y} = self.cell_index;
        let width = self.terminal.size().width as usize;
        let height = (self.terminal.size().height as usize)-1;
        let offset = &mut self.offset;
        //y is straight forward, one row for one terminal pixel
        if y < offset.y{
            offset.y = y;
        }
        else if y >= offset.y.saturating_add(height){
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        /* need to convert the length of row to the number of terminal pixels.
        This is to detemine how far and when to scroll */
        let mut strlen = 0;
        for i in offset.x..x+1{
            strlen += self.document.table.column_width(i);
            strlen += 4; //to offset added printer characters between lines
        }
        if strlen.saturating_sub(4) <= self.document.table.column_width(offset.x) && offset.x >= 1{
            offset.x = offset.x.saturating_sub(1);
        }
        else if strlen >= offset.x.saturating_add(width) {
            offset.x = offset.x.saturating_add(1);
        }
    }

    //does what is says it does
    fn move_position(&mut self, key: Key){
        let terminal_height = self.terminal.size().height as usize;
        let height = self.document.table.num_rows();
        let width = self.document.table.num_cols();
        let Position {mut x, mut y,} = self.cell_index;
        match key{
            Key::Up => {
                if y > 0{
                    y = y.saturating_sub(1)
                }
            } 
            Key::Down => {
                if y <= height{
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } 
            }
            Key::Right => {
                if x <= width {
                    x += 1;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height+1 {
                    y.saturating_sub(terminal_height)
                } else {
                    1
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                }
                else {
                    height-2
                }
            }
            Key::Home => x=1,
            Key::End => x = width,
            _ => {},
        }
        self.cell_index = Position{x , y}
        
    }

    //the rest of the code is just a bunch of string formatting to display data on the screen neatly
    fn draw_welcome_message(&self) 
    {
        let mut welcome_message = format!("CLICSV -- version: {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len)/2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("{}{}{}",(self.terminal.size().height/3).to_string(),spaces,welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn draw_status_bar(&self) 
    {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if !self.document.is_saved() 
        {
            " (modified)"
        } else 
        {
            ""
        };

        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name 
        {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
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

    fn draw_message_bar(&self)
    {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0)
        {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn draw_row(&self, ridx : u16){
        let ncols: usize = self.document.table.num_cols();
        let width: usize = self.terminal.size().width as usize;
        let row: Vec<&Cell> = self.document.get_row((ridx as usize)+self.offset.y-1);
        let mut row_str: String = String::new();
        let nrows: usize = self.document.table.num_rows();
        let mut diff: usize = 0;
        if row.len() != ncols{
            Terminal::clear_screen();
            println!("Error: rows have unequal amount of columns. Exiting...");
            std::process::exit(1);
        }
        for i in self.offset.x..ncols{
            let cell: &&Cell = &row[i];
            let s:String;
            let filling_width = self.document.table.column_width(cell.x_loc)-cell.width;
            if cell.highlighted{
                s = format!(
                    "{}{}{}{}{}{} {} ", 
                    color::Fg(STATUS_FG_COLOR),
                    color::Bg(STATUS_BG_COLOR),
                    cell.contents.clone(), 
                    &" ".repeat(filling_width),
                    color::Bg(color::Reset),
                    color::Fg(color::Reset),
                    "│");
                    diff += 45; //45 is the length added to string by fomatting color
            } else {
                s = format!(
                    "{}{} {} ", 
                    cell.contents.clone(), 
                    &" ".repeat(filling_width),
                    "│");
            }
            row_str = row_str.clone() + &s;
            if row_str.len() > width+diff{
                break;
            }
        }
        let len_term_str = (ridx as usize) + self.offset.y-2;
        let row_filling = nrows.to_string().len() - len_term_str.to_string().len();
        let terminal_row_str = String::from(len_term_str.to_string() + &" ".repeat(row_filling));
        let display_str = format!(
            "{}{}│{}{}\r",
            color::Fg(STATUS_FG_COLOR),
            terminal_row_str, 
            color::Fg(color::Reset),
            row_str
        );
        println!("{}\r",display_str);
    }

    fn draw_header(&self){
        let width: usize = self.terminal.size().width as usize;
        let ncols: usize = self.document.table.num_cols();
        let nrows: usize = self.document.table.num_rows();
        let mut col_str: String = String::new();
        (self.offset.x+1..ncols+1).for_each(|x| {
            let fill: usize = self.document.table.column_width(x)-1;
            col_str += &format!("{}{} {} ", num_to_let(x) ,&" ".repeat(fill), "|");
        });
        let row_fill: usize = nrows.to_string().len()+1;
        col_str = format!("{}{}{}",color::Fg(STATUS_FG_COLOR),String::from(&" ".repeat(row_fill)),&col_str.clone());
        col_str.truncate(width);
        println!("{}\r",col_str);
        Terminal::clear_current_line();
        println!("{}\r",&"-".repeat(width));
    }


    fn draw_table(&self){
        let height = self.terminal.size().height;
        let nrows = self.document.table.num_rows();
        Terminal::clear_current_line();
        self.draw_header();
        for terminal_row in 2..height {
            Terminal::clear_current_line();
            if terminal_row as usize <= nrows+1 && !self.document.is_empty(){            
                self.draw_row(terminal_row-1);
            }
            else if self.document.is_empty() && terminal_row == height/3{
                self.draw_welcome_message();
            }
            else
            {
                let edgenumber = terminal_row-2;
                println!("{}{}\r",color::Fg(STATUS_FG_COLOR),edgenumber.to_string());
            }
        }
    }

    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error>
    {
        let mut result = String::new();
        loop 
        {
            self.status_message = StatusMessage::from(format!("{}{}",prompt,result));
            self.refresh_screen()?;
            match Terminal::read_key()? 
            {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) => 
                {
                    if !c.is_control() 
                    {
                        result.push(c);
                    }

                }
                Key::Esc => 
                {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }   
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() 
        {
            return Ok(None);
        }
        Ok(Some(result))
    }


}
fn num_to_let(num: usize) -> char {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut idx = num;
    if num % 26 == 0{
        return 'A';
    }
    if 26 < num{
        let div = (num/26)*26 as usize;
        idx = num - div;
    }
    let c = alphabet.chars().nth(idx-1).unwrap();
    c
}

fn die(e: std::io::Error) 
{
    Terminal::clear_screen();
    panic!("{}\n",e);
}
