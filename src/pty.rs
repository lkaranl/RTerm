/// PTY Manager - Backend de shell assíncrono
/// Usa portable-pty com comunicação via channels

use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;

/// Mensagens do PTY para o terminal
pub enum PtyEvent {
    Output(Vec<u8>),
    Exit(i32),
}

/// Gerenciador do PTY
pub struct Pty {
    pair: PtyPair,
    writer: Box<dyn Write + Send>,
    pub rx: Receiver<PtyEvent>,
    _reader_thread: thread::JoinHandle<()>,
}

impl Pty {
    /// Cria um novo PTY com o shell padrão
    pub fn new(cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Detecta o shell padrão
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        
        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        
        // Spawn do processo filho
        let _child = pair.slave.spawn_command(cmd)?;
        
        // Configura comunicação
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        
        let (tx, rx): (Sender<PtyEvent>, Receiver<PtyEvent>) = bounded(1024);
        
        // Thread de leitura (não-bloqueante para o event loop)
        let reader_thread = thread::spawn(move || {
            Self::read_loop(reader, tx);
        });

        Ok(Self {
            pair,
            writer,
            rx,
            _reader_thread: reader_thread,
        })
    }

    /// Loop de leitura em thread separada
    fn read_loop(mut reader: Box<dyn Read + Send>, tx: Sender<PtyEvent>) {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = tx.send(PtyEvent::Exit(0));
                    break;
                }
                Ok(n) => {
                    let _ = tx.send(PtyEvent::Output(buf[..n].to_vec()));
                }
                Err(_) => {
                    let _ = tx.send(PtyEvent::Exit(1));
                    break;
                }
            }
        }
    }

    /// Envia input para o PTY
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Redimensiona o PTY
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.pair.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }
}
