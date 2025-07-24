use anyhow::Result;
use std::{
    io,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use crate::build;

pub fn watch() -> Result<()> {
    let websocket_server = TcpListener::bind("127.0.0.1:0")?;
    let websocket_port = websocket_server.local_addr()?.port();

    let sockets: Arc<Mutex<Vec<tungstenite::WebSocket<TcpStream>>>> =
        Arc::new(Mutex::new(Vec::new()));
    let sockets_clone = sockets.clone();
    let websocket_thread = thread::spawn(move || {
        for stream in websocket_server.incoming().flatten() {
            let websocket =
                tungstenite::accept(stream).expect("Failed to accept stream as websocket");
            sockets
                .lock()
                .expect("Failed to lock sockets")
                .push(websocket);
        }
    });

    let dist_dir = std::env::current_dir()?.join("dist");
    let serve_thread = thread::spawn(move || {
        let server = file_serve::ServerBuilder::new(&dist_dir)
            .hostname("0.0.0.0")
            .build();
        println!("Running on http://{}", server.addr());
        println!("Hit CTRL-C to stop");
        server.serve().expect("Server error");
    });

    pichu::watch(["content", "styles"], |_paths| {
        if let Err(e) = build(websocket_port.into(), false) {
            eprintln!("Build failed: {e}");
        }

        let mut sockets = sockets_clone.lock().expect("Failed to lock sockets");
        let mut broken = vec![];

        for (i, socket) in sockets.iter_mut().enumerate() {
            match socket.send("reload".into()) {
                Ok(()) => {}
                Err(tungstenite::error::Error::Io(e)) => {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        broken.push(i);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
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
    })?;

    websocket_thread
        .join()
        .expect("Failed to join websocket thread");
    serve_thread.join().expect("Failed to join serve thread");

    Ok(())
}
