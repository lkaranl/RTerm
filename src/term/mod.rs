/// Módulo do terminal
/// Contém grid de células e parser ANSI

pub mod grid;
pub mod ansi;

pub use grid::{Grid, Cell, CellStyle};
pub use ansi::AnsiParser;
