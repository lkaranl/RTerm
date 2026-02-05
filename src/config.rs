/// Configurações do RTerm
/// Otimizado para Apple Silicon (M4)

/// Dimensões padrão da janela
pub const DEFAULT_WIDTH: u32 = 1200;
pub const DEFAULT_HEIGHT: u32 = 800;

/// Configuração de fonte
pub const FONT_SIZE: f32 = 14.0;
pub const FONT_DATA: &[u8] = include_bytes!("/System/Library/Fonts/SFNSMono.ttf");

/// Dimensões de célula (calculadas em runtime baseado na fonte)
pub const CELL_WIDTH: f32 = 8.4;  // Aproximado para SF Mono 14pt
pub const CELL_HEIGHT: f32 = 17.0;

/// Cores padrão (Dracula-inspired)
pub const BG_COLOR: [f32; 4] = [0.157, 0.165, 0.212, 1.0];  // #282a36
pub const FG_COLOR: [f32; 4] = [0.973, 0.973, 0.949, 1.0];  // #f8f8f2

/// ANSI Colors (normal)
pub const ANSI_COLORS: [[f32; 4]; 16] = [
    [0.0, 0.0, 0.0, 1.0],           // 0: Black
    [1.0, 0.333, 0.333, 1.0],       // 1: Red
    [0.314, 0.980, 0.482, 1.0],     // 2: Green
    [0.945, 0.980, 0.549, 1.0],     // 3: Yellow
    [0.741, 0.576, 0.976, 1.0],     // 4: Blue
    [1.0, 0.475, 0.776, 1.0],       // 5: Magenta
    [0.545, 0.914, 0.992, 1.0],     // 6: Cyan
    [0.973, 0.973, 0.949, 1.0],     // 7: White
    [0.4, 0.435, 0.561, 1.0],       // 8: Bright Black
    [1.0, 0.333, 0.333, 1.0],       // 9: Bright Red
    [0.314, 0.980, 0.482, 1.0],     // 10: Bright Green
    [0.945, 0.980, 0.549, 1.0],     // 11: Bright Yellow
    [0.741, 0.576, 0.976, 1.0],     // 12: Bright Blue
    [1.0, 0.475, 0.776, 1.0],       // 13: Bright Magenta
    [0.545, 0.914, 0.992, 1.0],     // 14: Bright Cyan
    [1.0, 1.0, 1.0, 1.0],           // 15: Bright White
];

/// Performance settings
pub const TARGET_FPS: u32 = 120;  // ProMotion displays
pub const SCROLLBACK_LINES: usize = 10_000;
