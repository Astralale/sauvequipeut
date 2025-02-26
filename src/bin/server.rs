use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    // Écoute sur le port 3000
    let listener = TcpListener::bind("127.0.0.1:3000").expect("Impossible de démarrer le serveur");

    println!("Serveur en écoute sur http://127.0.0.1:3000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => eprintln!("Erreur de connexion : {}", e),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    // Lire la requête HTTP du client
    if let Ok(_) = stream.read(&mut buffer) {
        // Réponse HTTP basique
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, Rust!";
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
