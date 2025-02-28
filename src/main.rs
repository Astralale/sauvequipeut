use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use std::thread;
use std::time::Duration;

// Types pour les messages serveur
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
enum ServerMessage {
    RadarView { RadarView: String },
    Challenge { Challenge: ChallengeType },
}

#[derive(Serialize, Deserialize, Debug)]
enum ChallengeType {
    SecretSumModulo(u64),
    SOS,
}
#[derive(Clone, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    North,
    East,
    South,
    West,
}
struct GameState {
    secrets: Mutex<HashMap<String, u64>>, // Stocke les secrets des joueurs
}
struct PlayerState {
    position: Position,
    visited: HashMap<Position, u8>,
    last_direction: Option<&'static str>,
    orientation: Orientation, // 🔥 NOUVEAU : Orientation actuelle du joueur
}


impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
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
    pub fn compute_new_position(&self, movement: &str) -> Position {
        let mut new_pos = self.position.clone();

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

    pub fn update_orientation(&mut self, movement: &str) {
        match movement {
            "Left" => {
                self.orientation = match self.orientation {
                    Orientation::North => Orientation::West,
                    Orientation::South => Orientation::East,
                    Orientation::East => Orientation::North,
                    Orientation::West => Orientation::South,
                }
            }
            "Right" => {
                self.orientation = match self.orientation {
                    Orientation::North => Orientation::East,
                    Orientation::South => Orientation::West,
                    Orientation::East => Orientation::South,
                    Orientation::West => Orientation::North,
                }
            }
            "Back" => {
                self.orientation = match self.orientation {
                    Orientation::North => Orientation::South,
                    Orientation::South => Orientation::North,
                    Orientation::East => Orientation::West,
                    Orientation::West => Orientation::East,
                }
            }
            _ => {}
        }
    }
}


// Structures pour les messages client/serveur
#[derive(Serialize, Deserialize, Debug)]
struct RegisterTeam {
    RegisterTeam: RegisterTeamBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct RegisterTeamBody {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RegisterTeamResultWrapper {
    #[serde(rename = "RegisterTeamResult")]
    result: RegisterTeamResult,
}

#[derive(Serialize, Deserialize, Debug)]
enum RegisterTeamResult {
    #[serde(rename = "Ok")]
    Ok {
        expected_players: u8,
        registration_token: String,
    },
    #[serde(rename = "Err")]
    Err(String),
}

#[derive(Serialize, Deserialize, Debug)]
struct SubscribePlayer {
    SubscribePlayer: SubscribePlayerBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubscribePlayerBody {
    name: String,
    registration_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubscribePlayerResultWrapper {
    #[serde(rename = "SubscribePlayerResult")]
    result: SubscribePlayerResult,
}

#[derive(Serialize, Deserialize, Debug)]
enum SubscribePlayerResult {
    #[serde(rename = "Ok")]
    Ok,
    #[serde(rename = "Err")]
    Err(String),
}
#[derive(Debug)]
struct DecodedRadarView {
    horizontal_walls: Vec<u8>,
    vertical_walls: Vec<u8>,
    cells: Vec<u8>,
}


fn reconnect(server_address: &str) -> Result<TcpStream, String> {
    println!("Reconnecting to server...");
    TcpStream::connect(server_address).map_err(|e| e.to_string())
}

fn move_player(player_state: &mut PlayerState, movement: &str) {
    let mut new_pos = player_state.position.clone();

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
        },
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
        },
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
        },
        _ => (),
    }

    // ✅ **Mettre à jour la position**
    player_state.position = new_pos.clone();

    // ✅ **Enregistrer le passage sur cette case**
    let visit_count = player_state.visited.entry(new_pos.clone()).or_insert(0);
    if *visit_count < u8::MAX {
        *visit_count += 1;
    }

    println!(
        "[DEBUG] Nouvelle position: {:?}, Visites: {}",
        player_state.position, visit_count
    );
}





// Simule une boucle de jeu persistante après l'inscription des joueurs
fn main() -> Result<(), String> {
    let server_address = "127.0.0.1:3000";
    let team_name = "team_example";
    println!("Connecting to server at {}...", server_address);

    let mut stream = TcpStream::connect(server_address).map_err(|e| format!("Failed to connect: {}", e))?;
    println!("Connected to server!");

    let message = RegisterTeam {
        RegisterTeam: RegisterTeamBody {
            name: team_name.to_string(),
        },
    };

    let serialized_message = serde_json::to_string(&message).unwrap();
    let message_length = serialized_message.len() as u32;

    let mut buffer = vec![];
    buffer.extend(&message_length.to_le_bytes());
    buffer.extend(serialized_message.as_bytes());

    stream.write_all(&buffer).map_err(|e| format!("Failed to send message: {}", e))?;
    println!("RegisterTeam message sent!");

    let mut size_buffer = [0; 4];
    stream.read_exact(&mut size_buffer).map_err(|e| format!("Failed to read message size: {}", e))?;
    let response_size = u32::from_le_bytes(size_buffer) as usize;

    let mut response_buffer = vec![0; response_size];
    stream.read_exact(&mut response_buffer).map_err(|e| format!("Failed to read message: {}", e))?;
    let response = String::from_utf8_lossy(&response_buffer);
    println!("Raw server response: {}", response);

    let parsed_response: RegisterTeamResultWrapper = serde_json::from_str(&response).map_err(|e| format!("Failed to parse server response: {}", e))?;
    match parsed_response.result {
        RegisterTeamResult::Ok {
            expected_players,
            registration_token,
        } => {
            println!(
                "Team registered successfully! Expected players: {}, Registration token: {}",
                expected_players, registration_token
            );

            start_player_threads(server_address, registration_token, expected_players);
        }
        RegisterTeamResult::Err(err) => {
            println!("Failed to register team: {}", err);
        }
    }

    Ok(())
}

fn start_player_threads(server_address: &str, registration_token: String, expected_players: u8) {
    let mut handles = vec![];
    let game_state = Arc::new(GameState {
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

                let mut player_state = PlayerState {
                    position: Position::new(0, 0),
                    visited: HashMap::new(),
                    last_direction: None,
                    orientation: Orientation::North,
                };

                if let Err(e) = subscribe_player(&mut stream, &player_name, &token) {
                    eprintln!("Failed to register player {}: {}", player_name, e);
                    return;
                }

                println!("Player {} registered successfully!", player_name);
                start_game_loop(&mut stream, &player_name, &mut player_state,game_state);
            }});

        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.join() {
            eprintln!("A thread panicked: {:?}", e);
        }
    }
}

fn subscribe_player(stream: &mut TcpStream, player_name: &str, registration_token: &str) -> Result<(), String> {
    let message = serde_json::json!({
        "SubscribePlayer": {
            "name": player_name,
            "registration_token": registration_token,
        }
    });

    let serialized_message = serde_json::to_string(&message).unwrap();
    let message_length = serialized_message.len() as u32;

    let mut buffer = vec![];
    buffer.extend(&message_length.to_le_bytes());
    buffer.extend(serialized_message.as_bytes());

    stream.write_all(&buffer).map_err(|e| format!("Failed to send SubscribePlayer: {}", e))?;
    println!("SubscribePlayer message sent for {}", player_name);

    let mut size_buffer = [0; 4];
    stream.read_exact(&mut size_buffer).map_err(|e| format!("Failed to read SubscribePlayer response size: {}", e))?;
    let response_size = u32::from_le_bytes(size_buffer) as usize;

    let mut response_buffer = vec![0; response_size];
    stream.read_exact(&mut response_buffer).map_err(|e| format!("Failed to read SubscribePlayer response: {}", e))?;
    let response = String::from_utf8_lossy(&response_buffer);
    println!("SubscribePlayer response: {}", response);

    Ok(())
}
fn decode_b64(encoded: &str) -> Result<String, String> {
    let b64_alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/"
        .chars()
        .collect();

    let mut bits = String::new();
    for char in encoded.chars() {
        if let Some(index) = b64_alphabet.iter().position(|&c| c == char) {
            bits.push_str(&format!("{:06b}", index));
        } else {
            return Err(format!("Caractère non valide dans l'encodage b64 : {}", char));
        }
    }

    let bytes: Vec<u8> = bits
        .as_bytes()
        .chunks(8)
        .filter_map(|chunk| {
            if chunk.len() == 8 {
                Some(u8::from_str_radix(&String::from_utf8_lossy(chunk), 2).unwrap())
            } else {
                None
            }
        })
        .collect();

    let decoded = bytes
        .iter()
        .enumerate()
        .map(|(i, &byte)| {
            if i < 6 {
                format!("{:08b}", byte)
            } else {
                format!("{:02X}", byte)
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    Ok(decoded)
}


fn process_blocks(decoded: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
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
        horizontal_lines.chunks(3).map(|chunk| chunk.join(" ")).collect(),
        formatted_vertical_lines,
        formatted_cells,
    )
}

fn display_radar_view(horizontal: &[String], vertical: &[String], cells: &[String]) {
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
                    '0' => print!(" "),  // Rien (Empty)
                    '1' => print!("P"),  // Friendly player
                    '2' => print!("O"),  // Opponent
                    '3' => print!("M"),  // Monster
                    '4' => print!("H"),  // Hint
                    '8' => print!("G"),  // Goal
                    'B' => print!("*"),  // Goal with Player
                    'F' => print!("#"),  // Undefined
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

fn send_move_action(stream: &mut TcpStream, direction: &str,player_name: &str) -> Result<(), String> {
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

    println!("[{}] Move action sent: {}", player_name,direction);
    Ok(())
}


fn tremaux_decide_move(
    player_state: &mut PlayerState,
    radar_data: &[Vec<String>],
    cells: &[String],
    player_name: &str
) -> &'static str {
    let horizontal = &radar_data[0];
    let vertical = &radar_data[1];

    let current_pos = player_state.position.clone();

    // Mise à jour du compteur de visites
    let visit_count = player_state.visited.entry(current_pos.clone()).or_insert(0);
    if *visit_count < u8::MAX {
        *visit_count += 1;
    }

    println!(
        "[DEBUG {}] Position actuelle: {:?}, Visites: {}",
        player_name,current_pos, visit_count
    );

    let mut moves = Vec::new();
    let front_cell = cells.get(0).and_then(|row| row.chars().nth(1));
    let back_cell = cells.get(2).and_then(|row| row.chars().nth(1));
    let left_cell = cells.get(1).and_then(|row| row.chars().nth(0));
    let right_cell = cells.get(1).and_then(|row| row.chars().nth(2));
    println!(
        "[DEBUG {}] Cellule Value: Front: {:?}, Back: {:?}, Left: {:?}, Right: {:?}",
        player_name,front_cell, back_cell,left_cell,right_cell
    );
    if front_cell == Some('8')|| front_cell == Some('9'){
        println!("[DEBUG {}] 🚪 Sortie détectée devant ! Se dirige vers: Front", player_name);
        return "Front";
    }
    if back_cell == Some('8')|| front_cell == Some('9'){
        println!("[DEBUG {}] 🚪 Sortie détectée derrière ! Se dirige vers: Back", player_name);
        return "Back";
    }
    if left_cell == Some('8')|| front_cell == Some('9'){
        println!("[DEBUG {}] 🚪 Sortie détectée à gauche ! Se dirige vers: Left", player_name);
        return "Left";
    }
    if right_cell == Some('8')|| front_cell == Some('9'){
        println!("[DEBUG {}] 🚪 Sortie détectée à droite ! Se dirige vers: Right", player_name);
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
        moves.iter().map(|(dir, pos)| format!("{} -> {:?} (visites: {})", dir, pos, player_state.visited.get(pos).unwrap_or(&0))).collect::<Vec<_>>()
    );

    // **Prioriser la position avec le moins de visites**
    if let Some((best_move, _)) = moves.iter()
        .min_by_key(|(_, pos)| player_state.visited.get(pos).cloned().unwrap_or(0))
    {
        player_state.last_direction = Some(*best_move);
        return best_move;
    }


    //  Si aucune option viable, revenir en arrière**
    let last_option = moves.first().map(|(direction, _)| *direction).unwrap_or("Back");
    player_state.last_direction = Some(last_option);

    println!(
        "[DEBUG {}] Aucune option optimale, derniern option: {}",
        player_name, last_option
    );
    last_option
}


fn wall_follower_decide_move(player_state: &mut PlayerState, radar_data: &[Vec<String>], cells: &[String]) -> &'static str {
    let horizontal = &radar_data[0];
    let vertical = &radar_data[1];

    let current_pos = player_state.position.clone();

    // Stocker les mouvements possibles
    let mut moves = Vec::new();

    let possible_moves = vec![
        ("Right", vertical.get(1), 2, Position::new(current_pos.x + 1, current_pos.y)),
        ("Front", horizontal.get(1), 1, Position::new(current_pos.x, current_pos.y - 1)),
        ("Left", vertical.get(1), 0, Position::new(current_pos.x - 1, current_pos.y)),
        ("Back", horizontal.get(2), 1, Position::new(current_pos.x, current_pos.y + 1)),
    ];

    // Vérifier les passages ouverts
    for (direction, passage_option, index, new_pos) in &possible_moves {
        if let Some(passage) = passage_option {
            if let Some(middle_column) = passage.split_whitespace().nth(*index) {
                if middle_column == "01" {
                    moves.push((*direction, new_pos.clone()));
                }
            }
        }
    }

    // check si la sortie est visible
    for (i, row) in cells.iter().enumerate() {
        if let Some(index) = row.chars().position(|c| c == '8') {
            println!("[{:?}] Goal detected at row {}, column {}", player_state.position, i, index);
            return match i {
                0 => "Front",
                2 => "Back",
                _ => match index {
                    0 => "Left",
                    2 => "Right",
                    _ => "Front",
                },
            };
        }
    }

    // Filtre les mouvements déjà visités
    let mut best_move = None;
    let mut min_visits = u8::MAX;

    for (direction, new_pos) in &moves {
        let visits = player_state.visited.get(new_pos).copied().unwrap_or(0);
        if visits == 0 {
            return direction; // On prend direct une nouvelle case si elle existe !
        }
        if visits < min_visits {
            min_visits = visits;
            best_move = Some(direction);
        }
    }

    // Retourne le mouvement le moins visité ou recule sinon
    best_move.unwrap_or(&"Back")
}




fn handle_secret_sum_modulo(
    stream: &mut TcpStream,
    player_name: &str,
    game_state: &Arc<GameState>,
    modulo: u64,
) {
    // On verrouille le mutex UNE SEULE FOIS et on stocke les valeurs localement
    let known_secrets: Vec<u128> = {
        let secrets = game_state.secrets.lock().unwrap(); // On récupère TOUS les secrets de l'équipe
        secrets.values().map(|&v| v as u128).collect()
    };

    // Calcul de la somme des secrets
    let sum: u128 = known_secrets.iter().fold(0u128, |acc, &val| acc + val);

    println!("[{}] Known secrets before modulo: {:?}", player_name, known_secrets);
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

    let serialized_response = serde_json::to_string(&response).unwrap();
    let message_length = serialized_response.len() as u32;

    let mut buffer = vec![];
    buffer.extend(&message_length.to_le_bytes());
    buffer.extend(serialized_response.as_bytes());

    if let Err(e) = stream.write_all(&buffer) {
        eprintln!("[{}] Failed to send SecretSumModulo response: {}", player_name, e);
    } else {
        println!("[{}] Sent SolveChallenge response: {}", player_name, result);
    }
}








fn start_game_loop(stream: &mut TcpStream, player_name: &str, player_state: &mut PlayerState,game_state: Arc<GameState>) {
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
                            let direction = tremaux_decide_move(&mut player_state, &radar_data, &cells,player_name);
                            println!("[{}] Decided to move: {}", player_name, direction);
                            // Update de la position du joueur c'est pour tremaux
                            move_player(&mut player_state, direction);

                            send_move_action(stream, direction, player_name).unwrap_or_else(|e| {
                                eprintln!("[{}] Failed to send move action: {}", player_name, e);
                            });
                        }
                        Err(err) => eprintln!("[{}] Failed to decode RadarView: {}", player_name, err),
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
                        println!("[{}] SecretSumModulo challenge received with modulo {}", player_name, modulo_value);

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

