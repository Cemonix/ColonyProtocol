use super::utils::get_player_input;
use super::configs::player_names::{PlayerNameConfigError, generate_random_names};

#[derive(Debug, thiserror::Error)]
pub enum GameConfigurationError {
    #[error(transparent)]
    PlayerNameConfigError(#[from] PlayerNameConfigError)
}

pub enum MapSize {
    Small,
    Medium,
    Large
}

pub struct GameConfiguration {
    pub(crate) num_of_players: u8,
    pub(crate) player_names: Vec<String>,
    pub(crate) num_of_ai: u8,
    pub(crate) map_size: MapSize
}

impl GameConfiguration {
    pub fn new() -> Result<GameConfiguration, GameConfigurationError> {
        println!("\n=== COLONY PROTOCOL INITIALIZATION ===");
        println!("Establishing secure connection to Colonial Command...");
        println!("Connection established.\n");
    
        println!("QUERY: Number of human commanders in this sector (1-4):");
    
        let player_num = get_player_input(
            |input| {
                match input.parse::<u8>() {
                    Ok(p) if p >= 1 && p <= 4 => Ok(p),
                    Ok(_) => Err(String::from("Invalid parameter. Colonial doctrine allows 1-4 commanders.")),
                    Err(_) => Err(String::from("Invalid input format. Numerical value required."))
                }
            }
        );

        println!("QUERY: Designate commander identities manually? (y/N):");

        let name_players = get_player_input(
            |input| match input.to_lowercase().as_str() {
                "y" => Ok(true),
                "n" => Ok(false),
                _ => Err(String::from("Invalid response. Protocol requires affirmative (Y) or negative (N)."))
            }
        );

        let mut player_names: Vec<String> = Vec::with_capacity(player_num as usize);
        if name_players {
            for _ in 0..player_num {
                print!("Commander name");
                player_names.push(
                    get_player_input(
                        |input| Ok(String::from(input))
                    )
                );
            }
        }
        else {
            player_names = generate_random_names(player_num as usize)?;
        }
    
        println!("\nQUERY: Number of AI-controlled factions to deploy (0-4):");
    
        let ai_num = get_player_input(
            |input| {
                match input.parse::<u8>() {
                    Ok(a) if a <= 4 && (player_num + a) >= 2 => Ok(a),
                    Ok(_) => Err(String::from("Invalid parameter. Colonial doctrine allows 0-4 AI factions.")),
                    Err(_) => Err(String::from("Invalid input format. Numerical value required."))
                }
            }
        );
    
        println!("\nQUERY: Star system density configuration (small|medium|large):");
    
        let map_size = get_player_input(
            |input| {
                match input {
                    "small" => Ok(MapSize::Small),
                    "medium" => Ok(MapSize::Medium),
                    "large" => Ok(MapSize::Large),
                    _ => Err(String::from("Unknown configuration. Valid options: small, medium, large"))
                }
            }
        );
    
        println!("\n[INITIALIZING STAR SYSTEM...]");
        println!("[DEPLOYING COLONIAL FLEETS...]");
        println!("[ESTABLISHING QUANTUM LINKS...]");
        println!("\nColony Protocol active. Command interface ready.\n");
        
        Ok(
            GameConfiguration {
                num_of_players: player_num,
                player_names,
                num_of_ai: ai_num,
                map_size
            }
        )
    }
}