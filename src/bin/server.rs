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
    println!("Client connecté: {}", stream.peer_addr().unwrap());

    let mut buffer = [0; 1024]; // Buffer de lecture
    let expected_steps = ["login", "radar1", "radar2"]; // Étapes prévues
    let responses = ["OK", "action1", "action2"]; // Réponses du serveur

    for (step, response) in expected_steps.iter().zip(responses.iter()) {
        let size = stream.read(&mut buffer).unwrap();
        let received = String::from_utf8_lossy(&buffer[..size]).trim().to_string();

        if received != *step {
            println!("Erreur: attendu `{}`, reçu `{}`", step, received);
            break;
        }

        println!("Reçu `{}`, réponse `{}`", received, response);
        stream.write(response.as_bytes()).unwrap();
    }

    println!("Scénario terminé, fermeture de la connexion.");
}
