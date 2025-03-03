mod client;
mod game;
mod player;
mod utils;
mod config;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use crate::client::start_player_threads;
use crate::config::Config;

fn main() -> Result<(), String> {
    let config = Arc::new(Config::load().expect("Erreur chargement config"));
    println!("Configuration chargée: {:?}", config);

    let server_address = &config.server_address;
    let team_name = &config.team_name;

    println!("Connecting to server at {}...", server_address);
    let mut stream = TcpStream::connect(server_address)
        .map_err(|e| format!("Failed to connect: {}", e))?;
    println!("Connected to server!");

    let message = player::RegisterTeam {
        RegisterTeam: player::RegisterTeamBody {
            name: team_name.to_string(),
        },
    };

    let serialized_message = serde_json::to_string(&message)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;
    let message_length = serialized_message.len() as u32;

    let mut buffer = Vec::with_capacity(4 + serialized_message.len());
    buffer.extend(&message_length.to_le_bytes());
    buffer.extend(serialized_message.as_bytes());

    stream.write_all(&buffer)
        .map_err(|e| format!("Failed to send message: {}", e))?;
    println!("RegisterTeam message sent!");

    let mut size_buffer = [0; 4];
    stream.read_exact(&mut size_buffer)
        .map_err(|e| format!("Failed to read message size: {}", e))?;
    let response_size = u32::from_le_bytes(size_buffer) as usize;

    let mut response_buffer = vec![0; response_size];
    stream.read_exact(&mut response_buffer)
        .map_err(|e| format!("Failed to read message: {}", e))?;
    let response = String::from_utf8(response_buffer)
        .map_err(|e| format!("Invalid UTF-8 in response: {}", e))?;
    println!("Raw server response: {}", response);

    let parsed_response: player::RegisterTeamResultWrapper = serde_json::from_str(&response)
        .map_err(|e| format!("Failed to parse server response: {}", e))?;

    match parsed_response.result {
        player::RegisterTeamResult::Ok {
            expected_players,
            registration_token,
        } => {
            println!(
                "Team registered successfully! Expected players: {}, Registration token: {}",
                expected_players, registration_token
            );

            start_player_threads(server_address, registration_token, expected_players);
        }
        player::RegisterTeamResult::Err(err) => {
            eprintln!("Failed to register team: {}", err);
        }
    }

    Ok(())
}
