#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::net::TcpStream;
    use std::sync::{Arc, Mutex};
    use crate::game::GameState;
    use crate::player::{send_move_action, tremaux_decide_move, Orientation, PlayerState, Position};
    use crate::utils::decode_b64;

    #[test]
    fn test_compute_new_position() {
        let player = PlayerState {
            position: Position::new(0, 0),
            visited: HashMap::new(),
            last_direction: None,
            orientation: Orientation::North,
        };

        let new_position = player.compute_new_position("Front");
        assert_eq!(new_position, Position::new(0, -1));
    }

    #[test]
    fn test_update_orientation() {
        let mut player = PlayerState {
            position: Position::new(0, 0),
            visited: HashMap::new(),
            last_direction: None,
            orientation: Orientation::North,
        };

        player.update_orientation("Left");
        assert_eq!(player.orientation, Orientation::West);
    }

    #[test]
    fn test_handle_secret_sum_modulo() {
        let game_state = Arc::new(GameState {
            secrets: Mutex::new(HashMap::new()),
        });

        {
            let mut secrets = game_state.secrets.lock().unwrap();
            secrets.insert("player_1".to_string(), 11);
            secrets.insert("player_3".to_string(), 32);
        }

        let modulo = 10;
        let result = {
            let secrets = game_state.secrets.lock().unwrap();
            let sum: u128 = secrets.values().map(|&v| v as u128).sum();
            (sum % modulo as u128) as u64
        };

        assert_eq!(result, 3);
    }

    #[test]
    fn test_decode_b64() {
        let encoded = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";
        let decoded = decode_b64(encoded);
        assert!(decoded.is_ok());
    }

    #[test]
    fn test_server_connection() {
        let server_address = "127.0.0.1:8778";
        let connection = TcpStream::connect(server_address);
        assert!(connection.is_ok(), "La connexion au serveur a échoué !");
    }

    #[test]
    fn test_send_move_action() {
        let server_address = "127.0.0.1:8778";
        let mut stream = TcpStream::connect(server_address).expect("Échec de connexion");

        let result = send_move_action(&mut stream, "Front", "player_test");
        assert!(result.is_ok(), "Échec d'envoi de l'action de déplacement !");
    }

    #[test]
    fn test_tremaux_decide_move() {
        let mut player_state = PlayerState {
            position: Position::new(0, 0),
            visited: HashMap::new(),
            last_direction: None,
            orientation: Orientation::North,
        };

        let radar_data = vec![
            vec!["01 01 01".to_string()], // Passages horizontaux
            vec!["01 00 01".to_string()], // Passages verticaux
        ];
        let cells = vec!["000".to_string(), "000".to_string(), "000".to_string()];

        let move_direction = tremaux_decide_move(&mut player_state, &radar_data, &cells, "player_test");

        assert!(
            move_direction == "Front" || move_direction == "Right",
            "Le joueur ne devrait pas reculer immédiatement !"
        );
    }


}
