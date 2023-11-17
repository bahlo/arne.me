use anyhow::Result;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::{
    io,
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn watch() -> Result<()> {
    let websocket_server = TcpListener::bind("127.0.0.1:0")?;
    let websocket_port = websocket_server.local_addr()?.port();

    // Build on start
    crate::build(Some(websocket_port))?;

    let (notify_tx, notify_rx) = crossbeam_channel::unbounded::<DebounceEventResult>();
    let mut debouncer = new_debouncer(Duration::from_millis(500), notify_tx)?;

    let (build_tx, build_rx) = crossbeam_channel::unbounded::<()>();

    let build_thread = thread::spawn(move || {
        for rx in notify_rx {
            match rx {
                Ok(_events) => {
                    let mut child = match Command::new("cargo")
                        .arg("run")
                        .arg("build")
                        .args(&["--websocket-port", &websocket_port.to_string()])
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()
                    {
                        Ok(child) => child,
                        Err(e) => {
                            eprintln!("Error: {:?}", e);
                            return;
                        }
                    };

                    match child.wait() {
                        Ok(status) if status.success() => {
                            build_tx.send(()).unwrap();
                        }
                        Ok(status) => {
                            eprintln!("Error: Received status {:?}", status);
                        }
                        Err(e) => eprintln!("Error: {:?}", e),
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
    });

    let sockets: Arc<Mutex<Vec<tungstenite::WebSocket<TcpStream>>>> =
        Arc::new(Mutex::new(Vec::new()));
    let sockets_clone = sockets.clone();
    let websocket_thread = thread::spawn(move || {
        for stream in websocket_server.incoming() {
            let websocket = tungstenite::accept(stream.unwrap()).unwrap();
            sockets.lock().unwrap().push(websocket);
        }
    });

    let reload_thread = thread::spawn(move || {
        while build_rx.recv().is_ok() {
            let mut sockets = sockets_clone.lock().unwrap();
            let mut broken = vec![];

            for (i, socket) in sockets.iter_mut().enumerate() {
                match socket.send("reload".into()) {
                    Ok(_) => {}
                    Err(tungstenite::error::Error::Io(e)) => {
                        if e.kind() == io::ErrorKind::BrokenPipe {
                            broken.push(i);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                    }
                }
            }

            for i in broken.into_iter().rev() {
                sockets.remove(i);
            }

            // Close all but the last 10 connections
            let len = sockets.len();
            if len > 10 {
                for mut socket in sockets.drain(0..len - 10) {
                    socket.close(None).ok();
                }
            }
        }
    });

    debouncer
        .watcher()
        .watch(Path::new("./content"), RecursiveMode::Recursive)?;
    debouncer
        .watcher()
        .watch(Path::new("./src"), RecursiveMode::Recursive)?;
    debouncer
        .watcher()
        .watch(Path::new("./static"), RecursiveMode::Recursive)?;
    debouncer
        .watcher()
        .watch(Path::new("./styles"), RecursiveMode::Recursive)?;
    debouncer
        .watcher()
        .watch(Path::new("./Cargo.toml"), RecursiveMode::NonRecursive)?;
    debouncer
        .watcher()
        .watch(Path::new("./Cargo.lock"), RecursiveMode::NonRecursive)?;

    let dist = std::env::current_dir()?.join("dist");
    let server = file_serve::ServerBuilder::new(&dist)
        .hostname("0.0.0.0")
        .build();
    println!("Running on http://{}", server.addr());
    println!("Hit CTRL-C to stop");
    server.serve()?;

    build_thread.join().unwrap();
    reload_thread.join().unwrap();
    websocket_thread.join().unwrap();

    Ok(())
}
