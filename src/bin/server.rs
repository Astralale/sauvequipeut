use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use sauvequipeut::player;
use sauvequipeut::player::RegisterTeamBody;

fn main() {
    // Écoute sur le port 3000
    let listener = TcpListener::bind("127.0.0.1:8778").expect("Impossible de démarrer le serveur");

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
    println!("Client connecté: {}", stream.peer_addr().unwrap());

    let mut size_buffer = [0_u8; 4];
    stream.read_exact(&mut size_buffer).unwrap();
    let message_size = u32::from_le_bytes(size_buffer) as usize;

    let mut buffer = vec![0; message_size];
    stream.read_exact(&mut buffer).unwrap();

    let received = String::from_utf8_lossy(&buffer).trim().to_string();

    let mut writing_buffer = vec![];

    //Client Team response
    if received == get_expected_step(1) {
        writing_buffer.extend((get_response(1).len() as u32).to_le_bytes());
        writing_buffer.extend(get_response(1).as_bytes());

        println!("Reçu `{}`, réponse `{}`", received, get_response(2));
        stream.write_all(&writing_buffer).unwrap();
    }

    //Client Player response
    if received == get_expected_step(2) {
        writing_buffer.extend((get_response(2).len() as u32).to_le_bytes());
        writing_buffer.extend(get_response(2).as_bytes());

        println!("Reçu `{}`, réponse `{}`", received, get_response(2));
        stream.write_all(&writing_buffer).unwrap();

        writing_buffer = vec![];

        writing_buffer.extend((get_response(3).len() as u32).to_le_bytes());
        writing_buffer.extend(get_response(3).as_bytes());

        println!("Reçu `{}`, réponse `{}`", received, get_response(3));
        stream.write_all(&writing_buffer).unwrap();


        let mut size_buffer = [0_u8; 4];
        stream.read_exact(&mut size_buffer).unwrap();
        let message_size = u32::from_le_bytes(size_buffer) as usize;

        let mut buffer = vec![0; message_size];
        stream.read_exact(&mut buffer).unwrap();
        let received = String::from_utf8_lossy(&buffer).trim().to_string();
        println!("Reçu `{}`", received);
    }


}

fn get_expected_step(iteration: u8) -> String {
    match iteration {
        1 => {
            let register_team = player::RegisterTeam {
                RegisterTeam: player::RegisterTeamBody {
                    name: "team_example".to_string(),
                }
            };
            serde_json::to_string(&register_team).unwrap()
        },
        2 => {
            let subscribe_player = player::SubscribePlayer {
                SubscribePlayer: player::SubscribePlayerBody {
                    name: "player_1".to_string(),
                    registration_token: "abcd1234".to_string(),
                }
            };
            serde_json::to_string(&subscribe_player).unwrap()
        },
        3 => String::from("radar1"),
        _ => String::from(""),
    }
}

fn get_response(iteration: u8) -> String {
    match iteration {
        1 => {
            let register_team_result = player::RegisterTeamResult::Ok {
                expected_players: 1,
                registration_token: "abcd1234".to_string(),
            };
            serde_json::to_string(&serde_json::json!({"RegisterTeamResult": register_team_result})).unwrap()
        },
        2 => {
            let subscribe_player_result = player::SubscribePlayerResult::Ok {};
            serde_json::to_string(&serde_json::json!({"SubscribePlayerResult": subscribe_player_result})).unwrap()
        },
        3 => {
            serde_json::to_string(&serde_json::json!({"RadarView": "zveKvsuL8a8aaaa"})).unwrap()
        },
        _ => String::from(""),
    }
}