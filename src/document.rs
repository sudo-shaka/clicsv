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
    
        let mut table = Table::from(String::from(" "));
        table.cell_count = 0;
        Self{
            file_name: None,
            table: table,
            saved: false,
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

    pub fn copy(&mut self) -> Result<Vec<Cell>,Error> {
        let mut cells = Vec::new();
        for cell in &self.table.cells{
            if cell.highlighted{
                cells.push(cell.clone());
            }
        }
        Ok(cells)
    }

    pub fn paste(&mut self,at:&Position, cells: &Vec<Cell>) -> Result<(),Error> {
        self.saved = false;
        let mut x = at.x;
        let mut y = at.y;
        let mut prev_x = cells.first().unwrap().x_loc;
        let mut prev_y = cells.first().unwrap().y_loc;
       //only works for one cell at the moment 
        for cell in cells{         
            if cell.x_loc > prev_x{
                x +=1;
            }
            else if cell.y_loc > prev_y{
                y += 1;
            }
            let pos = Position {x,y};
            self.insert(pos,&cell.contents);
            prev_x = cell.x_loc;
            prev_y = cell.y_loc;
        }

        Ok(())
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

    pub fn save(&mut self) -> Result<(),Error>{
        if let Some(file_name) = &self.file_name{
            let mut file = fs::File::create(file_name)?;
            let n_rows = self.table.num_rows();
            let mut line = String::new();

            for i in 1..n_rows+1{
                for cell in &self.table.cells{
                    if i == cell.y_loc{
                        line.push_str(&cell.contents);
                        line.push_str(",");
                    }
                }
                line.pop();
                file.write_all(line.as_bytes())?;
                file.write_all(b"\n")?;
                line.clear();
            }
            self.saved = true;
        }
        Ok(())
    }

}
