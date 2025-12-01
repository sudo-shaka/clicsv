mod document;
mod editor;
mod table;
mod terminal;

pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use table::Table;
pub use terminal::Terminal;

fn main() {
    Editor::default().run();
}
