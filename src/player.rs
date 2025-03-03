use crate::game::GameState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write, Error};
use std::net::TcpStream;
use std::sync::Arc;
use std::fs::{File, OpenOptions};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    x: i32,
    y: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub position: Position,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

pub struct PlayerState {
    pub position: Position,
    pub visited: HashMap<Position, u8>,
    pub last_direction: Option<&'static str>,
    pub orientation: Orientation,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
/// Structure pour stocker l'historique des déplacements d'un joueur.
pub struct MovementLog {
    pub player_name: String,
}

impl MovementLog {
    /// Initialise le fichier de log en le vidant au début de chaque partie.
    pub fn reset_log(&self) -> Result<(), Error> {
        let filename = format!("{}_movements.log", self.player_name);
        File::create(filename)?; // Crée un fichier vide (écrase l'ancien)
        Ok(())
    }

    /// Enregistre un mouvement dans le fichier de log du joueur.
    ///
    /// # Arguments
    ///
    /// * `position` - La position actuelle du joueur après son déplacement.
    /// * `movement` - Le mouvement effectué ("Front", "Back", etc.).
    pub fn log_movement(&self, position: &Position, movement: &str) {
        let filename = format!("{}_movements.log", self.player_name);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .expect("Erreur lors de l'ouverture du fichier de log");

        writeln!(
            file,
            "Position: ({}, {}), Mouvement: {}",
            position.x, position.y, movement
        )
            .expect("Erreur lors de l'écriture du log");
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Position {}

impl std::hash::Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl PlayerState {
    /// Calcule la nouvelle position du joueur en fonction du mouvement spécifié.
    ///
    /// # Arguments
    ///
    /// * `movement` - Une chaîne représentant le mouvement à effectuer ("Front", "Back", "Left", "Right").
    ///
    /// # Retourne
    ///
    /// * `Position` - La nouvelle position après le déplacement.
    pub fn compute_new_position(&self, movement: &str) -> Position {
        let mut new_pos = self.position;

        match movement {
            "Front" => match self.orientation {
                Orientation::North => new_pos.y -= 1,
                Orientation::South => new_pos.y += 1,
                Orientation::East => new_pos.x += 1,
                Orientation::West => new_pos.x -= 1,
            },
            "Back" => match self.orientation {
                Orientation::North => new_pos.y += 1,
                Orientation::South => new_pos.y -= 1,
                Orientation::East => new_pos.x -= 1,
                Orientation::West => new_pos.x += 1,
            },
            "Left" => match self.orientation {
                Orientation::North => new_pos.x -= 1,
                Orientation::South => new_pos.x += 1,
                Orientation::East => new_pos.y -= 1,
                Orientation::West => new_pos.y += 1,
            },
            "Right" => match self.orientation {
                Orientation::North => new_pos.x += 1,
                Orientation::South => new_pos.x -= 1,
                Orientation::East => new_pos.y += 1,
                Orientation::West => new_pos.y -= 1,
            },
            _ => (),
        }

        new_pos
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RegisterTeam {
    pub RegisterTeam: RegisterTeamBody,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeamBody {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeamResultWrapper {
    #[serde(rename = "RegisterTeamResult")]
    pub result: RegisterTeamResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RegisterTeamResult {
    #[serde(rename = "Ok")]
    Ok {
        expected_players: u8,
        registration_token: String,
    },
    #[serde(rename = "Err")]
    Err(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct SubscribePlayer {
    pub SubscribePlayer: SubscribePlayerBody,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribePlayerBody {
    pub name: String,
    pub registration_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribePlayerResultWrapper {
    #[serde(rename = "SubscribePlayerResult")]
    result: SubscribePlayerResult,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribePlayerResult {
    #[serde(rename = "Ok")]
    Ok,
    #[serde(rename = "Err")]
    Err(String),
}

pub fn move_player(player_state: &mut PlayerState, movement: &str, logger: &MovementLog) {
    let mut new_pos = player_state.position;

    match movement {
        "Front" => match player_state.orientation {
            Orientation::North => new_pos.y -= 1,
            Orientation::South => new_pos.y += 1,
            Orientation::East => new_pos.x += 1,
            Orientation::West => new_pos.x -= 1,
        },
        "Back" => {
            match player_state.orientation {
                Orientation::North => new_pos.y += 1,
                Orientation::South => new_pos.y -= 1,
                Orientation::East => new_pos.x -= 1,
                Orientation::West => new_pos.x += 1,
            }
            // 🔥 **Changer l'orientation après déplacement**
            player_state.orientation = match player_state.orientation {
                Orientation::North => Orientation::South,
                Orientation::South => Orientation::North,
                Orientation::East => Orientation::West,
                Orientation::West => Orientation::East,
            };
        }
        "Left" => {
            match player_state.orientation {
                Orientation::North => new_pos.x -= 1,
                Orientation::South => new_pos.x += 1,
                Orientation::East => new_pos.y -= 1,
                Orientation::West => new_pos.y += 1,
            }
            player_state.orientation = match player_state.orientation {
                Orientation::North => Orientation::West,
                Orientation::South => Orientation::East,
                Orientation::East => Orientation::North,
                Orientation::West => Orientation::South,
            };
        }
        "Right" => {
            match player_state.orientation {
                Orientation::North => new_pos.x += 1,
                Orientation::South => new_pos.x -= 1,
                Orientation::East => new_pos.y += 1,
                Orientation::West => new_pos.y -= 1,
            }
            player_state.orientation = match player_state.orientation {
                Orientation::North => Orientation::East,
                Orientation::South => Orientation::West,
                Orientation::East => Orientation::South,
                Orientation::West => Orientation::North,
            };
        }
        _ => (),
    }

    player_state.position = new_pos;

    let visit_count = player_state.visited.entry(new_pos).or_insert(0);
    if *visit_count < u8::MAX {
         *visit_count = visit_count.saturating_add(1);
    }

    println!(
        "[DEBUG] Nouvelle position: {:?}, Visites: {}",
        player_state.position, visit_count
    );

    logger.log_movement(&player_state.position, movement);

}

pub fn subscribe_player(
    stream: &mut TcpStream,
    player_name: &str,
    registration_token: &str,
) -> Result<(), String> {
    let message = serde_json::json!({
        "SubscribePlayer": {
            "name": player_name,
            "registration_token": registration_token,
        }
    });

    let serialized_message = serde_json::to_string(&message)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;
    let message_length = serialized_message.len() as u32;

    let mut buffer = vec![];
    buffer.extend(&message_length.to_le_bytes());
    buffer.extend(serialized_message.as_bytes());

    stream
        .write_all(&buffer)
        .map_err(|e| format!("Failed to send SubscribePlayer: {}", e))?;
    println!("SubscribePlayer message sent for {}", player_name);

    let mut size_buffer = [0; 4];
    stream
        .read_exact(&mut size_buffer)
        .map_err(|e| format!("Failed to read SubscribePlayer response size: {}", e))?;
    let response_size = u32::from_le_bytes(size_buffer) as usize;

    let mut response_buffer = vec![0; response_size];
    stream
        .read_exact(&mut response_buffer)
        .map_err(|e| format!("Failed to read SubscribePlayer response: {}", e))?;
    let response = String::from_utf8_lossy(&response_buffer);
    println!("SubscribePlayer response: {}", response);

    Ok(())
}

pub fn process_blocks(decoded: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let parts: Vec<&str> = decoded.split(' ').collect();

    let horizontal_passages = &parts[0..3];
    let vertical_passages = &parts[3..6];
    let cells = &parts[6..];

    let inverted_horizontal = horizontal_passages
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<&str>>()
        .join(" ");
    let inverted_vertical = vertical_passages
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<&str>>()
        .join(" ");

    let parse_2bits = |binary_str: &str| -> Vec<String> {
        binary_str
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect()
    };

    let horizontal_lines: Vec<String> = inverted_horizontal
        .split_whitespace()
        .flat_map(parse_2bits)
        .collect();

    let vertical_lines: Vec<String> = inverted_vertical
        .split_whitespace()
        .flat_map(parse_2bits)
        .collect();

    let formatted_vertical_lines: Vec<String> = vertical_lines
        .chunks(4)
        .map(|chunk| chunk.join(" "))
        .collect();

    let concatenated_cells = cells.join("");
    let formatted_cells = vec![
        concatenated_cells[0..3].to_string(),
        concatenated_cells[3..6].to_string(),
        concatenated_cells[6..9].to_string(),
    ];

    (
        horizontal_lines
            .chunks(3)
            .map(|chunk| chunk.join(" "))
            .collect(),
        formatted_vertical_lines,
        formatted_cells,
    )
}

pub fn display_radar_view(horizontal: &[String], vertical: &[String], cells: &[String]) {
    println!("\nVisualisation de la radar view :\n");

    for i in 0..3 {
        // Afficher les passages horizontaux
        if i < horizontal.len() {
            let mut previous_passage = None;
            for passage in horizontal[i].split_whitespace() {
                match passage {
                    "00" => {
                        if let Some("01") | Some("10") = previous_passage {
                            print!("•#"); // Si la valeur précédente est "01" ou "10"
                        } else {
                            print!("##"); // Sinon
                        }
                    }
                    "01" => print!("• "), // Open
                    "10" => print!("•━"), // Wall
                    _ => print!("  "),
                }
                previous_passage = Some(passage);
            }
            println!("#");
        }

        // Afficher les passages verticaux et les cellules
        if i < vertical.len() && i < cells.len() {
            for (v, c) in vertical[i].split_whitespace().zip(cells[i].chars()) {
                match v {
                    "00" => print!("#"), // Undefined
                    "01" => print!(" "), // Open
                    "10" => print!("┃"), // Wall
                    _ => print!("#"),
                }
                match c {
                    '0' => print!(" "), // Rien (Empty)
                    '1' => print!("P"), // Friendly player
                    '2' => print!("O"), // Opponent
                    '3' => print!("M"), // Monster
                    '4' => print!("H"), // Hint
                    '8' => print!("G"), // Goal
                    'B' => print!("*"), // Goal with Player
                    'F' => print!("#"), // Undefined
                    _ => print!("#"),
                }
            }
            match vertical[i].split_whitespace().last() {
                Some("01") => println!(" "),
                Some("10") => println!("┃"),
                _ => println!("#"),
            }
        }
    }

    // Dernière ligne des passages horizontaux
    if let Some(last) = horizontal.last() {
        let mut previous_passage = None;
        for passage in last.split_whitespace() {
            match passage {
                "00" => {
                    if let Some("01") | Some("10") = previous_passage {
                        print!("•#"); // Si la valeur précédente est "01" ou "10"
                    } else {
                        print!("##"); // Sinon
                    }
                }
                "01" => print!("• "), // Open
                "10" => print!("•━"), // Wall
                _ => print!("##"),
            }
            previous_passage = Some(passage);
        }
        println!("#");
    }
}

/// Envoie une action de déplacement au serveur.
///
/// # Arguments
///
/// * `stream` - Une référence mutable vers le flux TCP.
/// * `direction` - La direction du déplacement ("Front", "Back", "Left", "Right").
/// * `player_name` - Le nom du joueur effectuant l'action.
///
/// # Retourne
///
/// * `Result<(), String>` - Un `Ok(())` si l'action est envoyée avec succès, sinon une `Err` avec un message d'erreur.
pub fn send_move_action(
    stream: &mut TcpStream,
    direction: &str,
    player_name: &str,
) -> Result<(), String> {
    let action = serde_json::json!({
        "Action": {
            "MoveTo": direction
        }
    });

    let serialized_message = serde_json::to_string(&action).map_err(|e| e.to_string())?;
    let message_length = serialized_message.len() as u32;

    // Vérifier la taille maximale du message avant d'envoyer
    if message_length > 1_048_576 {
        return Err(format!("Message size too large: {}", message_length));
    }

    let mut buffer = vec![];
    buffer.extend(&message_length.to_le_bytes()); // Taille en little-endian
    buffer.extend(serialized_message.as_bytes()); // Message JSON sérialisé
    stream.write_all(&buffer).map_err(|e| e.to_string())?;

    println!("[{}] Move action sent: {}", player_name, direction);
    Ok(())
}
/// Détermine le prochain mouvement du joueur en fonction de l'algorithme de Trémaux.
///
/// # Arguments
///
/// * `player_state` - État du joueur, contenant sa position et son historique de visites.
/// * `radar_data` - Données du radar indiquant les murs et les passages.
/// * `cells` - Informations sur les cellules adjacentes.
/// * `player_name` - Nom du joueur.
///
/// # Retourne
///
/// * Une `&'static str` indiquant la direction du mouvement ("Front", "Back", "Left", "Right").
pub fn tremaux_decide_move(
    player_state: &mut PlayerState,
    cells: &[String],
    player_name: &str,
) -> &'static str {
    let current_pos = player_state.position;

    // Mise à jour du compteur de visites
    let visit_count = player_state.visited.entry(current_pos).or_insert(0);
    if *visit_count < u8::MAX {
        *visit_count = visit_count.saturating_add(1);
    }

    println!(
        "[DEBUG {}] Position actuelle: {:?}, Visites: {}",
        player_name, current_pos, visit_count
    );

    let mut moves = Vec::new();
    let front_cell = cells.first().and_then(|row| row.chars().nth(1));
    let back_cell = cells.get(2).and_then(|row| row.chars().nth(1));
    let left_cell = cells.get(1).and_then(|row| row.chars().next());
    let right_cell = cells.get(1).and_then(|row| row.chars().nth(2));
    println!(
        "[DEBUG {}] Cellule Value: Front: {:?}, Back: {:?}, Left: {:?}, Right: {:?}",
        player_name, front_cell, back_cell, left_cell, right_cell
    );
    if front_cell == Some('8') || front_cell == Some('9') {
        println!(
            "[DEBUG {}] 🚪 Sortie détectée devant ! Se dirige vers: Front",
            player_name
        );
        return "Front";
    }
    if back_cell == Some('8') || front_cell == Some('9') {
        println!(
            "[DEBUG {}] 🚪 Sortie détectée derrière ! Se dirige vers: Back",
            player_name
        );
        return "Back";
    }
    if left_cell == Some('8') || front_cell == Some('9') {
        println!(
            "[DEBUG {}] 🚪 Sortie détectée à gauche ! Se dirige vers: Left",
            player_name
        );
        return "Left";
    }
    if right_cell == Some('8') || front_cell == Some('9') {
        println!(
            "[DEBUG {}] 🚪 Sortie détectée à droite ! Se dirige vers: Right",
            player_name
        );
        return "Right";
    }

    if front_cell == Some('0') {
        moves.push(("Front", player_state.compute_new_position("Front")));
    }
    if back_cell == Some('0') {
        moves.push(("Back", player_state.compute_new_position("Back")));
    }
    if left_cell == Some('0') {
        moves.push(("Left", player_state.compute_new_position("Left")));
    }
    if right_cell == Some('0') {
        moves.push(("Right", player_state.compute_new_position("Right")));
    }
    println!(
        "[DEBUG {}] Mouvements possibles: {:?}",
        player_name,
        moves
            .iter()
            .map(|(dir, pos)| format!(
                "{} -> {:?} (visites: {})",
                dir,
                pos,
                match player_state.visited.get(pos) {
                    Some(visits) => visits,
                    None => {
                        eprintln!("[DEBUG] Pas de visites enregistrées pour la position {:?}", pos);
                        &0
                    }
                }
            ))
            .collect::<Vec<_>>()
    );

    // **Prioriser la position avec le moins de visites**
    if let Some((best_move, _)) = moves
        .iter()
        .min_by_key(|(_, pos)| match player_state.visited.get(pos) {
            Some(visits) => *visits,
            None => {
                eprintln!("[DEBUG] Pas de visites enregistrées pour la position {:?}", pos);
                0
            }
        })
    {
        player_state.last_direction = Some(*best_move);
        return best_move;
    }

    let last_option = match moves.first() {
        Some((direction, _)) => *direction,
        None => {
            eprintln!("[DEBUG] Aucun mouvement viable, retour par défaut à Back");
            "Back"
        }
    };
    player_state.last_direction = Some(last_option);

    println!(
        "[DEBUG {}] Aucune option optimale, derniern option: {}",
        player_name, last_option
    );
    last_option
}

pub fn handle_secret_sum_modulo(
    stream: &mut TcpStream,
    player_name: &str,
    game_state: &Arc<GameState>,
    modulo: u64,
) {
    // On verrouille le mutex UNE SEULE FOIS et on stocke les valeurs localement
    let known_secrets: Vec<u128> = {
        let secrets = match game_state.secrets.lock() {
            Ok(secrets) => secrets,
            Err(e) => {
                eprintln!("[DEBUG] Échec du verrouillage du mutex des secrets : {}", e);
                return;
            }
        };
        secrets.values().map(|&v| v as u128).collect()
    };

    // Calcul de la somme des secrets
    let sum: u128 = known_secrets.iter().sum();

    println!(
        "[{}] Known secrets before modulo: {:?}",
        player_name, known_secrets
    );
    println!("[{}] Sum before modulo: {}", player_name, sum);
    println!("[{}] Modulo value: {}", player_name, modulo);

    // Appliquer le modulo en u128 et reconvertir en u64
    let result = (sum % modulo as u128) as u64;
    println!("[{}] SecretSumModulo result: {}", player_name, result);

    // envoi de la réponse au serveur la base
    let response = serde_json::json!({
        "Action": {
            "SolveChallenge": {
                "answer": result.to_string()
            }
        }
    });

    let serialized_response = match serde_json::to_string(&response) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("[DEBUG] Échec de la sérialisation de la réponse : {}", e);
            return;
        }
    };
    let message_length = serialized_response.len() as u32;

    let mut buffer = vec![];
    buffer.extend(&message_length.to_le_bytes());
    buffer.extend(serialized_response.as_bytes());

    if let Err(e) = stream.write_all(&buffer) {
        eprintln!(
            "[{}] Failed to send SecretSumModulo response: {}",
            player_name, e
        );
    } else {
        println!("[{}] Sent SolveChallenge response: {}", player_name, result);
    }
}
