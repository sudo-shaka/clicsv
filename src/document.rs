extern crate termion;
use crate::table;
use crate::Position;

use std::fs;
use std::io::{Error, Write};
use table::Table;
use table::Cell;


pub struct Document{
    pub file_name:Option<String>,
    pub table: Table,
    saved: bool,
}

impl Default for Document{
    fn default() -> Self{

        Self{
            file_name: None,
            table: Table::from(String::from(" ")),
            saved: false
        }
    }
}

impl Document{ 
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let table = Table::from(contents);

        Ok(Self{
            file_name: Some(filename.to_string()),
            table: table,
            saved: true,
        })

    }
    
    pub fn is_empty(&self)-> bool {
        self.table.cell_count == 0
    }

    pub fn is_saved(&self) -> bool{
        self.saved
    }

    pub fn len(&self) -> usize {
        self.table.cell_count
    }

    pub fn get_row(&self,index:usize) -> Vec<&Cell> {
        let mut row = Vec::new();
        for cell in &self.table.cells{
            if cell.y_loc == index{
                row.push(cell);
            }
        }
        row
    }

    pub fn insert_newrow(&mut self, at: &Position) {
        if at.y == self.table.num_rows() + 1{
            for i in 1..self.table.num_cols() +1 {
                let mut cell = Cell::from("");
                cell.y_loc = at.y;
                cell.x_loc = i;
                self.table.add(cell);
            }
            self.saved = false;
        }
        else{
            return;
        }
    }

    pub fn insert_newcol(&mut self, at: &Position){
        if at.x == self.table.num_cols() + 1{
            for i in 1..self.table.num_rows() + 1 {
                let mut cell = Cell::from(" ");
                cell.x_loc = at.x;
                cell.y_loc = i;
                self.table.add(cell);
            }
            self.saved = false;
        }
        else{
            return;
        }
    }

    pub fn highlight(&mut self, at: &Position){
        let cells = self.table.cells.clone();
        self.table.cells = Vec::new();
        for mut cell in cells{
            if cell.x_loc == at.x && cell.y_loc == at.y{
                cell.highlight();
            }
            else{
                cell.unhighlight();
            }
            self.table.cells.push(cell);
        }
    }

    pub fn insert(&mut self,at:Position,line: &str) {
        self.saved =false;
        let cells = self.table.cells.clone();
        self.table.cells = Vec::new();
        
        for c in cells{
            if c.x_loc == at.x && c.y_loc == at.y{
                let mut cell = Cell::from(line);
                cell.x_loc = at.x;
                cell.y_loc = at.y;
                self.table.cells.push(cell);
            }
            else{
                self.table.cells.push(c);
            }
        }
    }

    pub fn delete(&mut self, at: Position){
        self.saved = false;
        let cells = self.table.cells.clone();
        self.table.cells = Vec::new();
        for mut c in cells{
            if c.x_loc == at.x && c.y_loc == at.y{
                c.edit_content(String::from(""));
            }
            self.table.cells.push(c);
        }
    }

    pub fn save(&mut self) -> Result<(), Error>
    {
        if let Some(file_name) = &self.file_name 
        {
            let mut file = fs::File::create(file_name)?;
            let mut row_n = 1usize;
            let mut line = String::from("");
            for cell in &self.table.cells 
            {
                if row_n != cell.y_loc
                {
                    line.pop();
                    file.write_all(line.as_bytes())?;
                    file.write_all(b"\n")?;
                    line.clear();
                }
                line.push_str(&cell.contents);
                line.push_str(",");
                row_n = cell.y_loc;      
            }
            line.pop();
            file.write_all(line.as_bytes())?;
            self.saved = true;
        }
        Ok(())
    }

}
