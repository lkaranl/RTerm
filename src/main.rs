/// RTerm - Terminal de Alta Performance para Apple Silicon
/// GPU-accelerated via wgpu/Metal

mod config;
mod pty;
mod term;
mod renderer;

use anyhow::Result;
use crossbeam_channel::TryRecvError;
use std::sync::Arc;
use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

use config::{DEFAULT_WIDTH, DEFAULT_HEIGHT};
use pty::{Pty, PtyEvent};
use term::{Grid, AnsiParser};
use renderer::Renderer;

fn main() -> Result<()> {
    env_logger::init();
    
    let event_loop = EventLoop::new()?;
    
    // Cria a janela
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("RTerm")
            .with_inner_size(winit::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .build(&event_loop)?
    );

    // Inicializa renderer
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;
    
    // Calcula dimensões do grid
    let (cols, rows) = renderer.grid_dimensions();
    let mut grid = Grid::new(cols, rows);
    
    // Inicializa PTY
    let mut pty = Pty::new(cols as u16, rows as u16)?;
    let mut parser = AnsiParser::new();
    
    // Loop principal
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        // Processa output do PTY
        loop {
            match pty.rx.try_recv() {
                Ok(PtyEvent::Output(data)) => {
                    parser.process(&data, &mut grid);
                }
                Ok(PtyEvent::Exit(_)) => {
                    elwt.exit();
                    return;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    elwt.exit();
                    return;
                }
            }
        }
        
        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(physical_size);
                        let (cols, rows) = renderer.grid_dimensions();
                        grid.resize(cols, rows);
                        let _ = pty.resize(cols as u16, rows as u16);
                    }

                    WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            state: ElementState::Pressed,
                            logical_key,
                            text,
                            ..
                        },
                        ..
                    } => {
                        // Converte key para bytes
                        let bytes: Option<Vec<u8>> = match &logical_key {
                            Key::Named(NamedKey::Enter) => Some(vec![b'\r']),
                            Key::Named(NamedKey::Backspace) => Some(vec![0x7f]),
                            Key::Named(NamedKey::Tab) => Some(vec![b'\t']),
                            Key::Named(NamedKey::Escape) => Some(vec![0x1b]),
                            Key::Named(NamedKey::ArrowUp) => Some(b"\x1b[A".to_vec()),
                            Key::Named(NamedKey::ArrowDown) => Some(b"\x1b[B".to_vec()),
                            Key::Named(NamedKey::ArrowRight) => Some(b"\x1b[C".to_vec()),
                            Key::Named(NamedKey::ArrowLeft) => Some(b"\x1b[D".to_vec()),
                            Key::Named(NamedKey::Home) => Some(b"\x1b[H".to_vec()),
                            Key::Named(NamedKey::End) => Some(b"\x1b[F".to_vec()),
                            Key::Named(NamedKey::PageUp) => Some(b"\x1b[5~".to_vec()),
                            Key::Named(NamedKey::PageDown) => Some(b"\x1b[6~".to_vec()),
                            Key::Named(NamedKey::Delete) => Some(b"\x1b[3~".to_vec()),
                            _ => {
                                // Texto normal
                                text.as_ref().map(|t| t.as_bytes().to_vec())
                            }
                        };

                        if let Some(data) = bytes {
                            let _ = pty.write(&data);
                        }
                    }

                    WindowEvent::RedrawRequested => {
                        if let Err(e) = renderer.render(&grid) {
                            log::error!("Erro de renderização: {:?}", e);
                        }
                    }

                    _ => {}
                }
            }
            
            Event::AboutToWait => {
                window.request_redraw();
            }
            
            _ => {}
        }
    })?;
    
    Ok(())
}