/// Configurações do RTerm
/// Visual refinado com tema Catppuccin Mocha e tipografia otimizada

/// Dimensões padrão da janela
pub const DEFAULT_WIDTH: u32 = 1280;
pub const DEFAULT_HEIGHT: u32 = 850;

/// Configuração de fonte
pub const FONT_SIZE: f32 = 14.5;
/// Tenta carregar JetBrains Mono local, fallback para SF Mono do sistema
pub const FONT_DATA: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

/// Dimensões de célula
/// Ajustado para JetBrains Mono 14.5pt com bom line-height
pub const CELL_WIDTH: f32 = 9.0;
pub const CELL_HEIGHT: f32 = 22.0;  // Line height generoso (aprox 1.5x)

/// Padding interno generoso para respiro
pub const PADDING_X: f32 = 24.0;
pub const PADDING_Y: f32 = 24.0;

// ============================================================================
// TEMA: Catppuccin Mocha
// Alto contraste, cores vibrantes e fundo profundo
// ============================================================================

/// Cores do tema
pub const BG_COLOR: [f32; 4] = [0.118, 0.118, 0.180, 1.0];      // #1e1e2e
pub const FG_COLOR: [f32; 4] = [0.804, 0.840, 0.957, 1.0];      // #cdd6f4

/// Cores do cursor
pub const CURSOR_COLOR: [f32; 4] = [0.961, 0.878, 0.863, 1.0];  // #f5e0dc (Rosewater)
pub const CURSOR_TEXT_COLOR: [f32; 4] = [0.118, 0.118, 0.180, 1.0]; // #1e1e2e

/// Cores ANSI - Catppuccin Mocha Palette
pub const ANSI_COLORS: [[f32; 4]; 16] = [
    // Normal
    [0.275, 0.298, 0.368, 1.0],     // 0: Surface1  #45475a
    [0.953, 0.545, 0.659, 1.0],     // 1: Red       #f38ba8
    [0.651, 0.890, 0.631, 1.0],     // 2: Green     #a6e3a1
    [0.976, 0.890, 0.686, 1.0],     // 3: Yellow    #f9e2af
    [0.537, 0.706, 0.980, 1.0],     // 4: Blue      #89b4fa
    [0.961, 0.718, 0.898, 1.0],     // 5: Pink      #f5c2e7
    [0.580, 0.894, 0.925, 1.0],     // 6: Teal      #94e2d5
    [0.725, 0.765, 0.843, 1.0],     // 7: Subtext1  #bac2de
    // Bright
    [0.353, 0.376, 0.490, 1.0],     // 8: Surface2  #585b70
    [0.953, 0.545, 0.659, 1.0],     // 9: Red       #f38ba8
    [0.651, 0.890, 0.631, 1.0],     // 10: Green    #a6e3a1
    [0.976, 0.890, 0.686, 1.0],     // 11: Yellow   #f9e2af
    [0.537, 0.706, 0.980, 1.0],     // 12: Blue     #89b4fa
    [0.961, 0.718, 0.898, 1.0],     // 13: Pink     #f5c2e7
    [0.580, 0.894, 0.925, 1.0],     // 14: Teal     #94e2d5
    [0.655, 0.686, 0.776, 1.0],     // 15: Subtext0 #a6adc8
];

/// Performance settings
pub const SCROLLBACK_LINES: usize = 10_000;

/// Visual settings
pub const CURSOR_BLINK_RATE_MS: u64 = 700;  // Blink mais lento e suave
