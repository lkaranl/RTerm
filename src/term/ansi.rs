/// Parser ANSI de alta performance
/// State machine para sequências de escape

use crate::config::ANSI_COLORS;
use super::grid::{Grid, CellStyle};

/// Estados do parser
#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Ground,
    Escape,
    Csi,
    CsiParam,
    Osc,
}

/// Parser ANSI
pub struct AnsiParser {
    state: State,
    params: Vec<u16>,
    current_param: u16,
    intermediate: Vec<u8>,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            params: Vec::with_capacity(16),
            current_param: 0,
            intermediate: Vec::with_capacity(8),
        }
    }

    /// Processa bytes e atualiza o grid
    pub fn process(&mut self, data: &[u8], grid: &mut Grid) {
        for &byte in data {
            self.process_byte(byte, grid);
        }
    }

    fn process_byte(&mut self, byte: u8, grid: &mut Grid) {
        match self.state {
            State::Ground => self.ground(byte, grid),
            State::Escape => self.escape(byte, grid),
            State::Csi | State::CsiParam => self.csi(byte, grid),
            State::Osc => self.osc(byte, grid),
        }
    }

    fn ground(&mut self, byte: u8, grid: &mut Grid) {
        match byte {
            0x1b => self.state = State::Escape,
            0x07 => {} // Bell - ignorar
            0x08 => grid.backspace(),
            0x09 => grid.tab(),
            0x0a | 0x0b | 0x0c => grid.newline(),
            0x0d => grid.carriage_return(),
            0x20..=0x7e => grid.write_char(byte as char),
            0xc0..=0xff => {
                // UTF-8 multibyte - simplificado, renderiza como ?
                // TODO: implementar decode UTF-8 completo
                grid.write_char('?');
            }
            _ => {} // Ignora outros controles
        }
    }

    fn escape(&mut self, byte: u8, grid: &mut Grid) {
        match byte {
            b'[' => {
                self.state = State::Csi;
                self.params.clear();
                self.current_param = 0;
                self.intermediate.clear();
            }
            b']' => {
                self.state = State::Osc;
            }
            b'c' => {
                // Reset terminal
                grid.clear();
                grid.current_style = CellStyle::default();
                self.state = State::Ground;
            }
            b'D' => {
                // Index - move cursor down
                grid.newline();
                self.state = State::Ground;
            }
            b'E' => {
                // Next line
                grid.newline();
                grid.carriage_return();
                self.state = State::Ground;
            }
            b'M' => {
                // Reverse index - move cursor up
                if grid.cursor_y > 0 {
                    grid.cursor_y -= 1;
                }
                self.state = State::Ground;
            }
            _ => self.state = State::Ground,
        }
    }

    fn csi(&mut self, byte: u8, grid: &mut Grid) {
        match byte {
            b'0'..=b'9' => {
                self.state = State::CsiParam;
                self.current_param = self.current_param * 10 + (byte - b'0') as u16;
            }
            b';' => {
                self.params.push(self.current_param);
                self.current_param = 0;
            }
            b'?' | b'>' | b'!' => {
                self.intermediate.push(byte);
            }
            // Final bytes
            b'A' => {
                // Cursor up
                let n = self.get_param(0, 1) as isize;
                grid.move_cursor_relative(0, -n);
                self.reset();
            }
            b'B' => {
                // Cursor down
                let n = self.get_param(0, 1) as isize;
                grid.move_cursor_relative(0, n);
                self.reset();
            }
            b'C' => {
                // Cursor forward
                let n = self.get_param(0, 1) as isize;
                grid.move_cursor_relative(n, 0);
                self.reset();
            }
            b'D' => {
                // Cursor back
                let n = self.get_param(0, 1) as isize;
                grid.move_cursor_relative(-n, 0);
                self.reset();
            }
            b'H' | b'f' => {
                // Cursor position
                self.params.push(self.current_param);
                let row = self.get_param(0, 1).saturating_sub(1) as usize;
                let col = self.get_param(1, 1).saturating_sub(1) as usize;
                grid.move_cursor(col, row);
                self.reset();
            }
            b'J' => {
                // Erase in display
                self.params.push(self.current_param);
                match self.get_param(0, 0) {
                    0 => grid.clear_to_end_of_screen(),
                    1 => {} // TODO: clear from start
                    2 | 3 => grid.clear(),
                    _ => {}
                }
                self.reset();
            }
            b'K' => {
                // Erase in line
                self.params.push(self.current_param);
                match self.get_param(0, 0) {
                    0 => grid.clear_to_end_of_line(),
                    1 => {} // TODO: clear from start
                    2 => {
                        let y = grid.cursor_y;
                        grid.clear_line(y);
                    }
                    _ => {}
                }
                self.reset();
            }
            b'm' => {
                // SGR - Set Graphics Rendition
                self.params.push(self.current_param);
                self.process_sgr(grid);
                self.reset();
            }
            b'r' => {
                // Set scrolling region - ignorar por enquanto
                self.reset();
            }
            b'h' | b'l' => {
                // Set/reset mode - ignorar por enquanto
                self.reset();
            }
            b'c' => {
                // Device attributes - ignorar
                self.reset();
            }
            b'n' => {
                // Device status report - ignorar
                self.reset();
            }
            _ => {
                self.reset();
            }
        }
    }

    fn osc(&mut self, byte: u8, _grid: &mut Grid) {
        match byte {
            0x07 | 0x1b => {
                // OSC terminator - ignorar o conteúdo por enquanto
                self.state = State::Ground;
            }
            _ => {} // Acumular mas ignorar
        }
    }

    fn process_sgr(&mut self, grid: &mut Grid) {
        if self.params.is_empty() {
            grid.current_style = CellStyle::default();
            return;
        }

        let mut i = 0;
        while i < self.params.len() {
            match self.params[i] {
                0 => grid.current_style = CellStyle::default(),
                1 => grid.current_style.bold = true,
                3 => grid.current_style.italic = true,
                4 => grid.current_style.underline = true,
                7 => grid.current_style.inverse = true,
                22 => grid.current_style.bold = false,
                23 => grid.current_style.italic = false,
                24 => grid.current_style.underline = false,
                27 => grid.current_style.inverse = false,
                30..=37 => {
                    grid.current_style.fg = ANSI_COLORS[(self.params[i] - 30) as usize];
                }
                38 => {
                    // Extended foreground
                    if i + 2 < self.params.len() && self.params[i + 1] == 5 {
                        let color_idx = self.params[i + 2] as usize;
                        if color_idx < 16 {
                            grid.current_style.fg = ANSI_COLORS[color_idx];
                        }
                        i += 2;
                    } else if i + 4 < self.params.len() && self.params[i + 1] == 2 {
                        // True color
                        let r = self.params[i + 2] as f32 / 255.0;
                        let g = self.params[i + 3] as f32 / 255.0;
                        let b = self.params[i + 4] as f32 / 255.0;
                        grid.current_style.fg = [r, g, b, 1.0];
                        i += 4;
                    }
                }
                39 => grid.current_style.fg = crate::config::FG_COLOR,
                40..=47 => {
                    grid.current_style.bg = ANSI_COLORS[(self.params[i] - 40) as usize];
                }
                48 => {
                    // Extended background
                    if i + 2 < self.params.len() && self.params[i + 1] == 5 {
                        let color_idx = self.params[i + 2] as usize;
                        if color_idx < 16 {
                            grid.current_style.bg = ANSI_COLORS[color_idx];
                        }
                        i += 2;
                    } else if i + 4 < self.params.len() && self.params[i + 1] == 2 {
                        let r = self.params[i + 2] as f32 / 255.0;
                        let g = self.params[i + 3] as f32 / 255.0;
                        let b = self.params[i + 4] as f32 / 255.0;
                        grid.current_style.bg = [r, g, b, 1.0];
                        i += 4;
                    }
                }
                49 => grid.current_style.bg = crate::config::BG_COLOR,
                90..=97 => {
                    grid.current_style.fg = ANSI_COLORS[(self.params[i] - 90 + 8) as usize];
                }
                100..=107 => {
                    grid.current_style.bg = ANSI_COLORS[(self.params[i] - 100 + 8) as usize];
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn get_param(&self, idx: usize, default: u16) -> u16 {
        self.params.get(idx).copied().filter(|&v| v > 0).unwrap_or(default)
    }

    fn reset(&mut self) {
        self.state = State::Ground;
        self.params.clear();
        self.current_param = 0;
        self.intermediate.clear();
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}
