use crate::player::{
    display_radar_view, handle_secret_sum_modulo, move_player, process_blocks, send_move_action,
    tremaux_decide_move, Orientation, PlayerState, Position,
};
use crate::utils::decode_b64;
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct GameState {
    pub secrets: Mutex<HashMap<String, u64>>, // Stocke les secrets des joueurs
}

pub fn start_game_loop(
    stream: &mut TcpStream,
    player_name: &str,
    player_state: &mut PlayerState,
    game_state: Arc<GameState>,
) {
    let mut player_state = PlayerState {
        position: Position::new(0, 0),
        visited: HashMap::new(),
        last_direction: None, // Nouvelle initialisation
        orientation: Orientation::North,
    };
    loop {
        println!("[{}] Waiting for message...", player_name);

        let mut size_buffer = [0; 4];
        if let Err(e) = stream.read_exact(&mut size_buffer) {
            eprintln!("[{}] Failed to read message size: {}", player_name, e);
            break;
        }
        let message_size = u32::from_le_bytes(size_buffer) as usize;

        let mut response_buffer = vec![0; message_size];
        if let Err(e) = stream.read_exact(&mut response_buffer) {
            eprintln!("[{}] Failed to read message: {}", player_name, e);
            break;
        }

        let response: Result<serde_json::Value, _> = serde_json::from_slice(&response_buffer);
        match response {
            Ok(msg) => {
                if let Some(radar_view) = msg.get("RadarView") {
                    let encoded_view = radar_view.as_str().unwrap_or("");
                    println!("RadarView: {}", encoded_view);
                    match decode_b64(encoded_view) {
                        Ok(decoded_data) => {
                            println!("[{}] Decoded RadarView data: {}", player_name, decoded_data);

                            let (horizontal, vertical, cells) = process_blocks(&decoded_data);

                            println!("\nBlocs décodés :");
                            println!("[{}] Passages horizontaux : {:?}", player_name, horizontal);
                            println!("[{}] Passages verticaux : {:?}", player_name, vertical);
                            println!("[{}] Cellules : {:?}", player_name, cells);

                            display_radar_view(&horizontal, &vertical, &cells);

                            // decide du mouvement
                            let radar_data = vec![horizontal.clone(), vertical.clone()];
                            let direction = tremaux_decide_move(
                                &mut player_state,
                                &radar_data,
                                &cells,
                                player_name,
                            );
                            println!("[{}] Decided to move: {}", player_name, direction);
                            // Update de la position du joueur c'est pour tremaux
                            move_player(&mut player_state, direction);

                            send_move_action(stream, direction, player_name).unwrap_or_else(|e| {
                                eprintln!("[{}] Failed to send move action: {}", player_name, e);
                            });
                        }
                        Err(err) => {
                            eprintln!("[{}] Failed to decode RadarView: {}", player_name, err)
                        }
                    }
                }

                if let Some(hint) = msg.get("Hint") {
                    if let Some(secret) = hint.get("Secret") {
                        let secret_value = secret.as_u64().unwrap_or(0);
                        println!("[{}] Received secret: {}", player_name, secret_value);

                        // met à jour le secret dans l'état global partagé
                        {
                            let mut secrets = game_state.secrets.lock().unwrap();
                            secrets.insert(player_name.to_string(), secret_value);
                        }
                        {
                            let secrets = game_state.secrets.lock().unwrap();
                            println!("Current known secrets: {:?}", secrets);
                        }
                    }
                }

                if let Some(challenge) = msg.get("Challenge") {
                    if let Some(modulo) = challenge.get("SecretSumModulo") {
                        let modulo_value = modulo.as_u64().unwrap_or(1);
                        println!(
                            "[{}] SecretSumModulo challenge received with modulo {}",
                            player_name, modulo_value
                        );

                        handle_secret_sum_modulo(stream, player_name, &game_state, modulo_value);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "[{}] Failed to deserialize message: {:?}, error: {}",
                    player_name, response_buffer, e
                );
            }
        }
    }
}
