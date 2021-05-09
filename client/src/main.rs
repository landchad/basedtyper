use basedtyper::{
    event::*,
    app::App,
    handlers::{message_handler, input_handler},
    ui,
};

use std::{io, sync::{Arc, Mutex, mpsc}};

use crossterm::{ExecutableCommand, terminal::{enable_raw_mode, EnterAlternateScreen}};

use tui::{Terminal, backend::CrosstermBackend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen).unwrap();

    let (input_sender, input_receiver) = mpsc::channel::<String>();
    let (connection_sender, mut connection_receiver) = mpsc::channel::<String>();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new(1000);

    let mut app = App::new(terminal.size().unwrap());

    println!("\x1b[5 q");

    terminal.clear().unwrap();

    loop {
        if let Ok(msg) = connection_receiver.try_recv() {
            message_handler(msg, &mut app);
        }

        if let Ok(msg) = input_receiver.try_recv() {
            let args: Vec<&str> = msg.split(' ')
                .map(|elem| elem.trim())
                .collect();

            match args[0] {
                "connect" => {
                    let host = args[1];
                    let connection = app.connect(host.to_string());
                    if let Ok(conn) = connection {
                        connection_receiver = conn.1;
                        app.connection = Some(Arc::new(Mutex::new(conn.0)));
                    }
                }

                _ => ()
            }
        }

        terminal.draw(|f| ui::draw_ui(f, &app)).unwrap();

        if let Ok(Event::Input(event)) = events.next() {
            input_handler(event, &mut app, input_sender.clone(), connection_sender.clone());
        }

        if app.should_exit {
            break;
        }
    }

    app.exit().unwrap();
    Ok(())
}
