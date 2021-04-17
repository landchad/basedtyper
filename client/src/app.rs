use io::{Read, Write};
use tui::layout::{Constraint, Direction, Layout, Rect};
use crossterm::{execute, terminal::{LeaveAlternateScreen, disable_raw_mode}};
use serde_json::json;

use crate::ui::wordlist::Wordlist;

use super::config::Config;
use std::{io, net::TcpStream, path::Path, sync::mpsc::{self, Receiver}, time::Instant};

pub struct App {
    pub state: State,
    pub config: Config,
    pub input_string: String,
    pub time_taken: u128,
    pub timer: Option<Instant>,
    pub current_index: usize,
    pub current_error: String,
    pub should_exit: bool,
    pub wordlist: Wordlist,
    pub wordlist_name: String,
    pub host_name: String,
    pub host: (bool, String),
    pub chunks: Vec<Rect>,
}

pub enum State {
    MainMenu,
    EndScreen,
    Waiting,
    TypingGame,
    WordlistPrompt,
    HostPrompt,
}

impl App {
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Percentage(100)])
            .split(area);

        let config = Config::new();

        let (config, err) = if config.is_err() {
            (Config::default(), config.err().unwrap().to_string())
        } else { 
            (config.unwrap(), String::new()) 
        };

        Self {
            state: State::MainMenu,
            input_string: String::new(),
            timer: None,
            time_taken: 0,
            current_index: 1,
            config,
            current_error: err,
            should_exit: false,
            wordlist: Wordlist::new(Vec::new()),
            wordlist_name: String::new(),
            host_name: String::new(),
            host: (false, String::new()),
            chunks,
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn connect(&mut self, host: String) -> Result<Receiver<String>, std::io::Error> {
        let stream = TcpStream::connect(host);

        if let Err(e) = stream {
            self.current_error = e.to_string();
            return Err(e);
        }

        let (connection_sender, connection_receiver) = mpsc::channel::<String>();

        let mut stream = stream.unwrap();
        self.state = State::Waiting;

        let json = json!({
            "call": "init",
            "data": {
                "username": "bloatoo",
            }
        });

        stream.write_all(serde_json::to_string(&json).unwrap().as_bytes()).unwrap();

        std::thread::spawn(move || loop {
            let mut buf = vec![0u8; 1024];

            if stream.read(&mut buf).is_err() {
                break;
            }

            buf.retain(|byte| byte != &u8::MIN);

            if !buf.is_empty() {
                let data = String::from_utf8(buf).unwrap();
                connection_sender.send(data).unwrap();
            }
        });

        Ok(connection_receiver)
    }

    /*pub fn send_conn(&mut self, data: String) {
        if let Some(conn) = self.connection.clone() {
            conn.send(data).unwrap();
        }
    }*/
    
    pub fn restart(&mut self, state: State) {
        self.input_string = String::new();
        self.current_index = 1;
        self.time_taken = 0;
        self.current_error = String::new();
        self.host_name = String::new();

        match state {
            State::TypingGame => {
                self.timer = Some(Instant::now());
            }
            _ => self.timer = None
        }

        self.state = state;
    }

    pub fn start_timer(&mut self) {
        self.timer = Some(Instant::now());
    }

    pub fn exit(&self) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen, crossterm::cursor::Show)?;
        Ok(())
    }

    pub fn locate_wordlist(&self) -> String {
        let wordlist_name = if self.wordlist_name.ends_with(".basedtyper") {
            self.wordlist_name.to_string()
        } else {
            self.wordlist_name.clone() + ".basedtyper"
        };

        let path_str = &(self.config.general.wordlist_directory.clone() + "/" + &wordlist_name);
        let path = Path::new(path_str);

        let path = if path.is_file() {
            path.to_str().unwrap().to_string()
        } else {
            wordlist_name
        };

        path
    }

    pub fn decrement_index(&mut self) {
        if self.current_index - 1 > 0 {
            self.current_index -= 1;
        }
    }

    pub fn increment_index(&mut self) {
        self.current_index += 1;
    }
}
