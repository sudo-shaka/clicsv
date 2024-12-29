extern crate unicode_width;
use unicode_width::UnicodeWidthStr;

use crate::Position;


#[derive(PartialEq, Debug, Clone)]
pub struct Cell {
    pub contents: String,
    pub width: Width,
    pub x_loc: usize,
    pub y_loc: usize,
    pub highlighted: bool,
}

impl From<String> for Cell {
    fn from(string:String) -> Self{
        Self {
            width:UnicodeWidthStr::width(&*string),
            contents: string,
            x_loc: 0usize,
            y_loc: 0usize,
            highlighted: false,
        }
    }
}

impl <'a> From<&'a str> for Cell {
    fn from(string: &'a str) -> Self{
        Self {
            width: UnicodeWidthStr::width(&*string),
            contents: string.into(),
            x_loc: 0usize,
            y_loc: 0usize,
            highlighted: false,
        }
    }
}

impl Cell {
    pub fn filling_width(self, maximum_width: Width) -> Width {
        self.width-maximum_width+1
    }
    pub fn edit_content(&mut self, new_content: String){
        self.contents = new_content.clone();
        self.width = new_content.len();
    }
    pub fn highlight(&mut self) {
        self.highlighted = true;
    }
    pub fn unhighlight(&mut self){
        self.highlighted = false;
    }
    pub fn get_content(self) -> String{
        self.contents
    }
}


pub type Width = usize;

#[derive(PartialEq, Debug, Default)]
pub struct Table {
    pub cells: Vec<Cell>,
    pub widest_cell_length: Width,
    pub width_sum: Width,
    pub cell_count: usize,
}

impl From<String> for Table
{
    fn from(slice: String) -> Self 
    {
        let mut cells = Vec::new();
        let mut y = 0usize;
        let mut cell_count = 0usize;
        let mut widest_cell_length = 0usize;
        let mut width_sum = 0usize;
        
        for value in slice.lines()
        {
            y += 1;
            let mut j = 0usize;
            let mut line = String::from(value);
            if line.len() > width_sum 
            {
                width_sum = line.len()
            }
            line.push(',');
            let mut x = 0usize;
            for (i, c) in line.chars().enumerate() 
            {
                if c == ',' 
                {
                    x +=1;
                    let mut cell = Cell::from(String::from(&line[j..i])+&" ");
                    cell_count += 1;
                    cell.x_loc = x;
                    cell.y_loc = y;
                    if cell.width > widest_cell_length
                    {
                        widest_cell_length = cell.width;
                    }                  
                    cells.push(cell);
                    j = i+1;
                }
            }
        }
        Self 
        {
            cells: cells,
            widest_cell_length: widest_cell_length,
            width_sum: width_sum,
            cell_count: cell_count,
        }
    }
}

impl Table{
    pub fn new() -> Self{
        let cells = Vec::new();
        Self {
            cells, 
            widest_cell_length: 0, 
            width_sum: 0, 
            cell_count: 0
        }
    }

    pub fn column_width(&self, x_loc: usize) -> Width {
        let mut width = 0usize;
        for cell in &self.cells {
            if cell.x_loc == x_loc{
                if cell.width > width{
                    width = cell.width;
                }
            }
        }
        width
    }

    pub fn row_width(&self) -> Width {
        self.width_sum + 2*self.num_cols() + self.num_rows().to_string().len()+1
    }

    pub fn get_content_from(&self, at: Position) -> String {
        for cell in &self.cells{
            if cell.x_loc == at.x && cell.y_loc == at.y{
                return cell.contents.clone();
            }
        }
        return "".to_string();
    }

    pub fn add(&mut self, cell: Cell) {
        if cell.width > self.widest_cell_length {
            self.widest_cell_length = cell.width;
        }
        self.width_sum += cell.width;
        self.cell_count += 1;
        self.cells.push(cell);
    }

    pub fn filling_width(&self, maximum_width: Width, cell_width: Width) -> Width{
        cell_width-maximum_width
    }

    pub fn num_rows(&self) -> usize {
        let mut num_line = 0usize;
        for cell in &self.cells {
            if cell.y_loc > num_line{
                num_line = cell.y_loc;
            }
        }
        num_line
    }

    pub fn num_cols(&self) -> usize {
        let mut num_col = 0usize;
        for cell in &self.cells {
            if cell.x_loc > num_col {
                num_col = cell.x_loc;
            }
        }
        num_col
    }

}