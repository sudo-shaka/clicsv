extern crate termion;
use crate::table;
use crate::Position;

use calamine::{open_workbook_auto, DataType, Reader};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use std::fs;
use std::fs::File;
use std::io::{Error, Read, Write};
use table::Cell;
use table::Table;
use termion::event::Key;
use zip::read::ZipArchive;

pub struct Action {
    pub key: Key,
    pub cells_affected: Vec<Cell>,
}

pub struct Document {
    pub file_name: Option<String>,
    pub table: Table,
    saved: bool,
    pub last_action: Action,
}

impl Default for Document {
    fn default() -> Self {
        let mut table = Table::from(String::from(" "));
        table.cell_count = 0;
        Self {
            file_name: None,
            table: table,
            saved: false,
            last_action: Action {
                key: Key::Null,
                cells_affected: Vec::new(),
            },
        }
    }
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        // If it's an ODS file, read content.xml inside the zip and parse table rows
        let table = if filename.ends_with(".ods") {
            let file = File::open(filename)?;
            let mut archive = ZipArchive::new(file)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let mut content_file = archive.by_name("content.xml").map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "No content.xml in .ods")
            })?;
            let mut content = String::new();
            content_file.read_to_string(&mut content)?;

            let mut reader = XmlReader::from_str(&content);
            reader.trim_text(true);
            let mut buf = Vec::new();

            let mut lines: Vec<String> = Vec::new();
            let mut current_row: Vec<String> = Vec::new();
            let mut current_cell = String::new();
            let mut in_p = false;

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) => {
                        if let Ok(name) = std::str::from_utf8(e.name().as_ref()) {
                            if name.ends_with("table-row") {
                                current_row = Vec::new();
                            } else if name.ends_with("table-cell") {
                                current_cell.clear();
                            } else if name.ends_with("p") || name.ends_with("text:p") {
                                in_p = true;
                            }
                        }
                    }
                    Ok(Event::Text(e)) => {
                        if in_p {
                            if let Ok(s) = e.unescape().map(|cow| cow.into_owned()) {
                                current_cell.push_str(&s);
                            }
                        }
                    }
                    Ok(Event::End(ref e)) => {
                        if let Ok(name) = std::str::from_utf8(e.name().as_ref()) {
                            if name.ends_with("p") || name.ends_with("text:p") {
                                in_p = false;
                            } else if name.ends_with("table-cell") {
                                current_row.push(current_cell.clone());
                            } else if name.ends_with("table-row") {
                                lines.push(current_row.join(","));
                            }
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
                    _ => {}
                }
                buf.clear();
            }

            Table::from(lines.join("\n"))
        } else if filename.ends_with(".xlsx") || filename.ends_with(".xls") {
            let mut workbook = open_workbook_auto(filename)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let sheet_names = workbook.sheet_names().to_owned();
            if sheet_names.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Excel workbook contains no sheets",
                ));
            }

            let first_sheet = sheet_names[0].clone();
            let range = workbook
                .worksheet_range(&first_sheet)
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Unable to read sheet")
                })
                .and_then(|r| r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))?;

            let mut lines: Vec<String> = Vec::new();
            for row in range.rows() {
                let mut cells: Vec<String> = Vec::new();
                for cell in row.iter() {
                    let s = match cell {
                        DataType::String(v) => v.clone(),
                        DataType::Float(v) => v.to_string(),
                        DataType::Int(v) => v.to_string(),
                        DataType::Bool(v) => v.to_string(),
                        DataType::Empty => String::new(),
                        other => format!("{}", other),
                    };
                    cells.push(s);
                }
                lines.push(cells.join(","));
            }

            Table::from(lines.join("\n"))
        } else {
            let contents = fs::read_to_string(filename)?;
            Table::from(contents)
        };

        Ok(Self {
            file_name: Some(filename.to_string()),
            table: table,
            saved: true,
            last_action: Action {
                key: Key::Null,
                cells_affected: Vec::new(),
            },
        })
    }

    pub fn is_empty(&self) -> bool {
        self.table.cell_count == 0
    }

    pub fn is_saved(&self) -> bool {
        self.saved
    }

    pub fn len(&self) -> usize {
        self.table.cell_count
    }

    pub fn get_row(&self, index: usize) -> Vec<Cell> {
        let mut row: Vec<Cell> = Vec::new();
        let ncols = self.table.num_cols();

        for x in 1..=ncols {
            // find a cell at (x, index)
            let mut found: Option<Cell> = None;
            for cell in &self.table.cells {
                if cell.x_loc == x && cell.y_loc == index {
                    found = Some(cell.clone());
                    break;
                }
            }
            if let Some(c) = found {
                row.push(c);
            } else {
                let mut empty = Cell::from(" ");
                empty.x_loc = x;
                empty.y_loc = index;
                row.push(empty);
            }
        }

        row
    }

    pub fn insert_newrow(&mut self, at: &Position) {
        if at.y == self.table.num_rows() + 1 {
            for i in 1..self.table.num_cols() + 1 {
                let mut cell = Cell::from("");
                cell.y_loc = at.y;
                cell.x_loc = i;
                self.table.add(cell);
            }
            self.saved = false;
        } else {
            return;
        }
    }

    pub fn insert_newcol(&mut self, at: &Position) {
        if at.x == self.table.num_cols() + 1 {
            for i in 1..self.table.num_rows() + 1 {
                let mut cell = Cell::from(" ");
                cell.x_loc = at.x;
                cell.y_loc = i;
                self.table.add(cell);
            }
            self.saved = false;
        } else {
            return;
        }
    }

    pub fn highlight(&mut self, at: &Position) {
        let cells = self.table.cells.clone();
        self.table.cells = Vec::new();
        for mut cell in cells {
            if cell.x_loc == at.x && cell.y_loc == at.y {
                cell.highlight();
            } else {
                cell.unhighlight();
            }
            self.table.cells.push(cell);
        }
    }

    pub fn multi_highlight(&mut self, at: &Position) {
        let cells = self.table.cells.clone();
        self.table.cells = Vec::new();
        for mut cell in cells {
            if cell.x_loc == at.x && cell.y_loc == at.y {
                cell.highlight();
            }
            self.table.cells.push(cell);
        }
    }

    pub fn copy(&mut self) -> Result<Vec<Cell>, Error> {
        let mut cells = Vec::new();
        for cell in &self.table.cells {
            if cell.highlighted {
                cells.push(cell.clone());
            }
        }
        Ok(cells)
    }

    pub fn get_highlight_cells(&self) -> Vec<Cell> {
        let mut cells = Vec::new();
        for c in &self.table.cells {
            if c.highlighted {
                cells.push(c.clone());
            }
        }
        return cells;
    }

    pub fn undo(&mut self) {
        if self.last_action.key == Key::Null {
            return;
        }
        for cell in self.last_action.cells_affected.clone() {
            let pos = Position {
                x: cell.x_loc,
                y: cell.y_loc,
            };
            self.insert(&pos, &cell.contents);
        }
    }

    pub fn paste(&mut self, at: &Position, cells: &Vec<Cell>) -> Result<(), Error> {
        self.saved = false;
        self.last_action.cells_affected = Vec::new();
        let mut x = at.x;
        let mut y = at.y;
        let mut prev_x = cells.first().unwrap().x_loc;
        let mut prev_y = cells.first().unwrap().y_loc;
        if x == 0 {
            x = 1;
        }
        if y == 0 {
            y = 1;
        }
        for cell in cells {
            if cell.x_loc > prev_x {
                x += 1;
            } else if cell.y_loc > prev_y {
                y += 1;
            }
            let mut c = cell.clone();
            c.contents = self.table.get_content_from(Position { x, y });
            c.x_loc = x;
            c.y_loc = y;
            self.last_action.cells_affected.push(c);
            self.insert(&Position { x, y }, &cell.contents);
            prev_x = cell.x_loc;
            prev_y = cell.y_loc;
        }

        Ok(())
    }

    pub fn insert(&mut self, at: &Position, line: &str) {
        self.saved = false;
        let cells = self.table.cells.clone();
        self.table.cells = Vec::new();

        for c in cells {
            if c.x_loc == at.x && c.y_loc == at.y {
                let mut cell = Cell::from(line);
                cell.x_loc = at.x;
                cell.y_loc = at.y;
                self.table.cells.push(cell);
            } else {
                self.table.cells.push(c);
            }
        }
    }

    pub fn delete(&mut self) {
        let cells = self.table.cells.clone();
        self.table.cells = Vec::new();
        self.saved = false;
        for mut c in cells {
            if c.highlighted {
                c.edit_content(String::from(" "));
            }
            self.table.cells.push(c);
        }
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            // If original file was Excel/ODS, save as new CSV file instead
            let mut target_name = file_name.clone();
            if file_name.ends_with(".xlsx")
                || file_name.ends_with(".xls")
                || file_name.ends_with(".ods")
            {
                if let Some(pos) = file_name.rfind('.') {
                    target_name = format!("{}.csv", &file_name[..pos]);
                } else {
                    target_name = format!("{}.csv", file_name);
                }
                // update stored file_name to the new csv so subsequent saves write to it
                self.file_name = Some(target_name.clone());
            }

            let mut file = fs::File::create(&target_name)?;
            let n_rows = self.table.num_rows();
            let mut line = String::new();

            for i in 1..n_rows + 1 {
                for cell in &self.table.cells {
                    if i == cell.y_loc {
                        line.push_str(&cell.contents);
                        line.pop();
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
