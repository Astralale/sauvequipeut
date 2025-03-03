use crate::player::{display_radar_view, handle_secret_sum_modulo, move_player, process_blocks, random_decide_move, send_move_action, tremaux_decide_move, MovementLog, Orientation, PlayerState, Position};
use crate::utils::decode_b64;
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use crate::config::Config;

pub struct GameState {
    pub secrets: Mutex<HashMap<String, u64>>, // Stocke les secrets des joueurs
}

pub fn start_game_loop(
    stream: &mut TcpStream,
    player_name: &str,
    game_state: Arc<GameState>,
) {
    let config = Arc::new(Config::load().expect("Erreur chargement config"));
    let mut player_state = PlayerState {
        position: Position::new(0, 0),
        visited: HashMap::new(),
        last_direction: None, // Nouvelle initialisation
        orientation: Orientation::North,
    };
    let movement_logger = MovementLog {
        player_name: player_name.to_string(),
    };
    if let Err(e) = movement_logger.reset_log() {
        eprintln!("[{}] Erreur lors de la réinitialisation du log : {}", player_name, e);
    }
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
                    if let Some(encoded_view) = radar_view.as_str() {
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

                                let direction = match config.navigation_mode.as_str() {
                                    "random" => random_decide_move(&cells, player_name),
                                    _ => tremaux_decide_move(&mut player_state, &cells, player_name),
                                };


                                println!("[{}] Decided to move: {}", player_name, direction);
                                move_player(&mut player_state, direction, &movement_logger);

                                if let Err(e) = send_move_action(stream, direction, player_name) {
                                    eprintln!("[{}] Failed to send move action: {}", player_name, e);
                                }
                            }
                            Err(err) => {
                                eprintln!("[{}] Failed to decode RadarView: {}", player_name, err);
                            }
                        }
                    }
                }

                if let Some(hint) = msg.get("Hint") {
                    if let Some(secret) = hint.get("Secret") {
                        if let Some(secret_value) = secret.as_u64() {
                            println!("[{}] Received secret: {}", player_name, secret_value);
                            if let Ok(mut secrets) = game_state.secrets.lock() {
                                secrets.insert(player_name.to_string(), secret_value);
                            } else {
                                eprintln!("[{}] Failed to lock secrets mutex", player_name);
                            }
                        }
                    }
                }

                if let Some(challenge) = msg.get("Challenge") {
                    if let Some(modulo) = challenge.get("SecretSumModulo") {
                        if let Some(modulo_value) = modulo.as_u64() {
                            println!(
                                "[{}] SecretSumModulo challenge received with modulo {}",
                                player_name, modulo_value
                            );

                            handle_secret_sum_modulo(stream, player_name, &game_state, modulo_value);
                        }
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
