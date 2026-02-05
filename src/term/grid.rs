/// Grid de células do terminal
/// Buffer duplo para renderização eficiente

use crate::config::{SCROLLBACK_LINES, FG_COLOR, BG_COLOR};

/// Estilo de uma célula
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellStyle {
    pub fg: [f32; 4],
    pub bg: [f32; 4],
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            fg: FG_COLOR,
            bg: BG_COLOR,
            bold: false,
            italic: false,
            underline: false,
            inverse: false,
        }
    }
}

/// Uma célula no grid
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub c: char,
    pub style: CellStyle,
    pub dirty: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            style: CellStyle::default(),
            dirty: true,
        }
    }
}

/// Grid do terminal com scrollback
pub struct Grid {
    /// Células visíveis
    cells: Vec<Vec<Cell>>,
    /// Scrollback buffer
    scrollback: Vec<Vec<Cell>>,
    /// Dimensões
    pub cols: usize,
    pub rows: usize,
    /// Posição do cursor
    pub cursor_x: usize,
    pub cursor_y: usize,
    /// Estilo atual
    pub current_style: CellStyle,
    /// Flag de dirty global
    pub dirty: bool,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = vec![vec![Cell::default(); cols]; rows];
        
        Self {
            cells,
            scrollback: Vec::with_capacity(SCROLLBACK_LINES),
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            current_style: CellStyle::default(),
            dirty: true,
        }
    }

    /// Escreve um caractere na posição do cursor
    pub fn write_char(&mut self, c: char) {
        if self.cursor_x >= self.cols {
            self.newline();
        }
        
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            self.cells[self.cursor_y][self.cursor_x] = Cell {
                c,
                style: self.current_style,
                dirty: true,
            };
            self.cursor_x += 1;
            self.dirty = true;
        }
    }

    /// Nova linha
    pub fn newline(&mut self) {
        self.cursor_x = 0;
        if self.cursor_y + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.cursor_y += 1;
        }
    }

    /// Carriage return
    pub fn carriage_return(&mut self) {
        self.cursor_x = 0;
    }

    /// Scroll up uma linha
    fn scroll_up(&mut self) {
        // Move primeira linha para scrollback
        if self.scrollback.len() >= SCROLLBACK_LINES {
            self.scrollback.remove(0);
        }
        let first_line = self.cells.remove(0);
        self.scrollback.push(first_line);
        
        // Adiciona nova linha vazia no final
        self.cells.push(vec![Cell::default(); self.cols]);
        self.dirty = true;
    }

    /// Backspace
    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.cells[self.cursor_y][self.cursor_x] = Cell::default();
            self.dirty = true;
        }
    }

    /// Tab
    pub fn tab(&mut self) {
        let next_tab = (self.cursor_x / 8 + 1) * 8;
        self.cursor_x = next_tab.min(self.cols - 1);
    }

    /// Limpa a tela
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.dirty = true;
    }

    /// Limpa do cursor até o fim da linha
    pub fn clear_to_end_of_line(&mut self) {
        for x in self.cursor_x..self.cols {
            self.cells[self.cursor_y][x] = Cell::default();
        }
        self.dirty = true;
    }

    /// Limpa do cursor até o fim da tela
    pub fn clear_to_end_of_screen(&mut self) {
        self.clear_to_end_of_line();
        for y in (self.cursor_y + 1)..self.rows {
            for x in 0..self.cols {
                self.cells[y][x] = Cell::default();
            }
        }
        self.dirty = true;
    }

    /// Limpa uma linha específica
    pub fn clear_line(&mut self, y: usize) {
        if y < self.rows {
            for x in 0..self.cols {
                self.cells[y][x] = Cell::default();
            }
            self.dirty = true;
        }
    }

    /// Move o cursor
    pub fn move_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(self.cols.saturating_sub(1));
        self.cursor_y = y.min(self.rows.saturating_sub(1));
    }

    /// Move cursor relativo
    pub fn move_cursor_relative(&mut self, dx: isize, dy: isize) {
        let new_x = (self.cursor_x as isize + dx).max(0) as usize;
        let new_y = (self.cursor_y as isize + dy).max(0) as usize;
        self.move_cursor(new_x, new_y);
    }

    /// Retorna uma célula
    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y][x]
    }

    /// Redimensiona o grid
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let mut new_cells = vec![vec![Cell::default(); cols]; rows];
        
        // Copia células existentes
        for y in 0..rows.min(self.rows) {
            for x in 0..cols.min(self.cols) {
                new_cells[y][x] = self.cells[y][x];
            }
        }
        
        self.cells = new_cells;
        self.cols = cols;
        self.rows = rows;
        self.cursor_x = self.cursor_x.min(cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(rows.saturating_sub(1));
        self.dirty = true;
    }

    /// Marca tudo como limpo
    pub fn mark_clean(&mut self) {
        self.dirty = false;
        for row in &mut self.cells {
            for cell in row {
                cell.dirty = false;
            }
        }
    }
}
