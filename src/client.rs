use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::{game, player};
use crate::game::start_game_loop;

pub fn start_player_threads(
    server_address: &str,
    registration_token: String,
    expected_players: u8
) {
    let mut handles = vec![];
    let game_state = Arc::new(game::GameState {
        secrets: Mutex::new(HashMap::new()),
    });

    for i in 1..=expected_players {
        let player_name = format!("player_{}", i);
        let token = registration_token.clone();
        let server = server_address.to_string();

        let handle = thread::spawn({
            let game_state = game_state.clone();
            move || {
                println!("Thread started for {}", player_name);

                let mut stream = match TcpStream::connect(&server) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to connect for {}: {}", player_name, e);
                        return;
                    }
                };

                if let Err(e) = player::subscribe_player(&mut stream, &player_name, &token) {
                    eprintln!("Failed to register player {}: {}", player_name, e);
                    return;
                }

                println!("Player {} registered successfully!", player_name);
                start_game_loop(&mut stream, &player_name, game_state);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.join() {
            eprintln!("A thread panicked: {:?}", e);
        }
    }
}
