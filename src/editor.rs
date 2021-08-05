use crate::Document;
use crate::Terminal;
use std::env;
use std::time::{Duration, Instant};
use termion::{color, event::Key};

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63,63,63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Debug, PartialEq, Clone)]
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
    cursor_position: Position,
    cell_index: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
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
            if !file_name.ends_with(".csv"){
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
            cursor_position: Position::default(),
            cell_index: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
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
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
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
            Key::Ctrl('s') => self.save(),
            Key::Char(c) => {
                if c == '\n'{
                    let content = self.prompt("INSERT: ").unwrap_or(None);
                    if content.is_none(){
                        self.status_message = StatusMessage::from(format!("Not Saved"));
                    }
                    else
                    {
                        let pos = self.cell_index.clone();
                        self.document.insert(pos,&content.unwrap());
                    }
                }
                else{
                    
                }

                return Ok(());
            }
            Key::Delete =>{
                let pos = self.cell_index.clone();
                self.document.delete(pos);
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => (self.move_cursor(pressed_key)),
            _ => (),
        }
        if self.cell_index.y > self.document.table.num_rows(){
            self.document.insert_newrow(&self.cell_index);
        }
        if self.cell_index.x > self.document.table.num_cols(){
            self.document.insert_newcol(&self.cell_index);
        }

        self.document.highlight(&self.cell_index.clone());
        self.scroll();
        Ok(())
    }

    /*fn cell_index_to_cursor_position(&mut self) -> Position {

        let Position{mut x,mut y} = self.cell_index;
        let mut width = 0usize;        

        for i in 1..x {
            width += self.document.table.column_width(i)+1;
        }
        width += self.document.table.num_rows().to_string().len();
        y += 1;
        x = width;
        
        Position {x,y}
    }*/


    //this needs work
    fn cursor_position_to_cell_index(&mut self) -> Position {
        let Position {mut x,mut y,} = self.cursor_position;
        let row_string_width = self.document.table.num_rows().to_string().len();
        let mut spaces = 0usize;

        if x <= row_string_width {
            x=0;
        }
        else{
            for i in 1..self.document.table.num_cols(){
                if spaces < x-row_string_width{
                    spaces += self.document.table.column_width(i)+2;
                }
                else{
                    x = i-1;
                    break;
                }
            }
        }

        if y > 0{
            y -= 1;
        }
        Position {x,y}
    }

    fn scroll(&mut self){
        let Position {x , y} = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y{
            offset.y = y;
        }
        else if y >= offset.y.saturating_add(height){
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x{
            offset.x = x;
        }
        else if x >= offset.x.saturating_add(width){
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key){
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x,} = self.cursor_position;
        let height = self.document.table.num_rows()+1;
        //let width = self.document.table.row_width();
        let width = self.terminal.size().width as usize;

        match key {
            Key::Up => {
                if y > 0{
                    y = y.saturating_sub(1)
                }
            }
            Key::Down => {
                if y <= height {
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
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                }
                else {
                    height
                }
            }
            Key::Home => x=0,
            Key::End => x = width,
            _ => {},
        } 
        
        if x > self.terminal.size().width as usize{
            x = width;
        }
        
        self.cursor_position = Position {x, y};
        self.cell_index = self.cursor_position_to_cell_index();
        //self.highlight();
    }

    fn draw_welcome_message(&self) 
    {
        let mut welcome_message = format!("CSVEDIT -- version: {}", VERSION);
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
            "{}/{}{}",
            self.cursor_position.y.saturating_add(1),
            self.document.table.num_rows(),
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



    fn draw_table(&self){
        /*
        This functionality works for far to cleanly display the rows but repeating functions to 
        convert to strings to get the lengths to determine the buffer size needed to properly display the rows is not ideal.
        Cleaner functionality needs to be implemented here...
        */
        let height = self.terminal.size().height;
        let width = self.terminal.size().width;
        for terminal_row in 1..height - 1 {
            if terminal_row == 1u16{
                Terminal::clear_current_line();
                let mut col_str = String::new();
                for x in 1..self.document.table.num_cols()+1 {
                    let fill = self.document.table.column_width(x)-terminal_row.to_string().len();
                    let cs = format!("{}{} {} ",num_to_let(x),&" ".repeat(fill), "│");
                    col_str = col_str.clone() + &cs;
                }
                let row_fill = self.document.table.num_rows().to_string().len()+1;
                col_str = format!("{}{}{}",color::Fg(STATUS_FG_COLOR),String::from(&" ".repeat(row_fill)),&col_str.clone());
                col_str.truncate(width as usize);
                println!("{}\r",col_str);
                Terminal::clear_current_line();
                println!("{}\r",&"-".repeat(width as usize));
            }
            if terminal_row as usize <= self.document.table.num_rows() && !self.document.is_empty(){            
                Terminal::clear_current_line();
                let row = self.document.get_row(terminal_row as usize);
                let mut row_str = String::new();
                for cell in row{
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
                    }
                    else{
                        s = format!(
                            "{}{} {} ", 
                            cell.contents.clone(), 
                            &" ".repeat(filling_width),
                            "│");
                    }
                    row_str = row_str.clone() + &s;
                }
                let row_filling = self.document.table.num_rows().to_string().len() - terminal_row.to_string().len();
                let terminal_row_str = String::from(terminal_row.to_string() + &" ".repeat(row_filling));
                let mut display_str = format!(
                    "{}{}│{}{}\r",
                    color::Fg(STATUS_FG_COLOR),
                    terminal_row_str, 
                    color::Fg(color::Reset),
                    row_str
                );
                display_str.truncate(width as usize);
                println!("{}\r",display_str);
            }
            else if self.document.is_empty() && terminal_row == height/3{
                self.draw_welcome_message();
            }
            else
            {
                Terminal::clear_current_line();
                println!("{}{}\r",color::Fg(STATUS_FG_COLOR),terminal_row.to_string());
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
    let div = num/26 as usize;
    let num = num - 26*div;
    let c = alphabet.chars().nth(num-1).unwrap();
    c
}

fn die(e: std::io::Error) 
{
    Terminal::clear_screen();
    panic!("{}\n",e);
}
