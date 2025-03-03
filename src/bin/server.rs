use sauvequipeut::player;
use sauvequipeut::player::RegisterTeamBody;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:8778") {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Impossible de démarrer le serveur : {}", e);
            return;
        }
    };

    println!("Serveur en écoute sur http://127.0.0.1:8778 ");

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
    match stream.peer_addr() {
        Ok(addr) => println!("Client connecté: {}", addr),
        Err(e) => {
            eprintln!("Impossible d'obtenir l'adresse du client : {}", e);
            return;
        }
    }

    let mut size_buffer = [0_u8; 4];
    if let Err(e) = stream.read_exact(&mut size_buffer) {
        eprintln!("Erreur lors de la lecture de la taille du message : {}", e);
        return;
    }
    let message_size = u32::from_le_bytes(size_buffer) as usize;

    let mut buffer = vec![0; message_size];
    if let Err(e) = stream.read_exact(&mut buffer) {
        eprintln!("Erreur lors de la lecture du message : {}", e);
        return;
    }

    let received = String::from_utf8_lossy(&buffer).trim().to_string();

    let mut writing_buffer = vec![];

    //Client Team response
    if received == get_expected_step(1) {
        writing_buffer.extend((get_response(1).len() as u32).to_le_bytes());
        writing_buffer.extend(get_response(1).as_bytes());

        println!("Reçu `{}`, réponse `{}`", received, get_response(2));
        if let Err(e) = stream.write_all(&writing_buffer) {
            eprintln!("Erreur lors de l'envoi de la réponse : {}", e);
            return;
        }
    }

    //Client Player response
    if received == get_expected_step(2) {
        writing_buffer.extend((get_response(2).len() as u32).to_le_bytes());
        writing_buffer.extend(get_response(2).as_bytes());

        println!("Reçu `{}`, réponse `{}`", received, get_response(2));
        if let Err(e) = stream.write_all(&writing_buffer) {
            eprintln!("Erreur lors de l'envoi de la réponse : {}", e);
            return;
        }

        writing_buffer = vec![];

        writing_buffer.extend((get_response(3).len() as u32).to_le_bytes());
        writing_buffer.extend(get_response(3).as_bytes());

        println!("Reçu `{}`, réponse `{}`", received, get_response(3));
        if let Err(e) = stream.write_all(&writing_buffer) {
            eprintln!("Erreur lors de l'envoi de la réponse : {}", e);
            return;
        }

        let mut size_buffer = [0_u8; 4];
        if let Err(e) = stream.read_exact(&mut size_buffer) {
            eprintln!("Erreur lors de la lecture de la taille du message : {}", e);
            return;
        }
        let message_size = u32::from_le_bytes(size_buffer) as usize;

        let mut buffer = vec![0; message_size];
        if let Err(e) = stream.read_exact(&mut buffer) {
            eprintln!("Erreur lors de la lecture du message : {}", e);
            return;
        }
        let received = String::from_utf8_lossy(&buffer).trim().to_string();
        println!("Reçu `{}`", received);
    }
}

fn get_expected_step(iteration: u8) -> String {
    match iteration {
        1 => match serde_json::to_string(&player::RegisterTeam {
            RegisterTeam: player::RegisterTeamBody {
                name: "team_example".to_string(),
            },
        }) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Erreur de sérialisation de RegisterTeam : {}", e);
                String::new()
            }
        },
        2 => match serde_json::to_string(&player::SubscribePlayer {
            SubscribePlayer: player::SubscribePlayerBody {
                name: "player_1".to_string(),
                registration_token: "abcd1234".to_string(),
            },
        }) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Erreur de sérialisation de SubscribePlayer : {}", e);
                String::new()
            }
        },
        3 => String::from("radar1"),
        _ => String::new(),
    }
}

fn get_response(iteration: u8) -> String {
    match iteration {
        1 => match serde_json::to_string(
            &serde_json::json!({"RegisterTeamResult": player::RegisterTeamResult::Ok {
                expected_players: 1,
                registration_token: "abcd1234".to_string(),
            }}),
        ) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Erreur de sérialisation de RegisterTeamResult : {}", e);
                String::new()
            }
        },
        2 => match serde_json::to_string(
            &serde_json::json!({"SubscribePlayerResult": player::SubscribePlayerResult::Ok {}}),
        ) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Erreur de sérialisation de SubscribePlayerResult : {}", e);
                String::new()
            }
        },
        3 => match serde_json::to_string(&serde_json::json!({"RadarView": "zveKvsuL8a8aaaa"})) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Erreur de sérialisation de RadarView : {}", e);
                String::new()
            }
        },
        _ => String::new(),
    }
}
