use crate::commands::parser::Parseable;
use crate::game_state::GameState;
use crate::commands::command::{CommandEffect, CommandError};
use crate::planet::PlanetId;

pub enum StatusTarget {
    Turn,
    Planets,
    Planet { id: PlanetId },
    Player,
}

pub struct StatusArgs {
    pub target: StatusTarget
}

impl Parseable for StatusArgs {
    fn parse(args: Vec<&str>) -> Result<Self, CommandError> {
        if args.is_empty() {
            return Err(CommandError::MissingArguments {
                command: String::from("status"),
                expected: String::from("status <planets|planet <id>|player>"),
            });
        }

        let target = match args[0] {
            "turn" => StatusTarget::Turn,
            "planets" => StatusTarget::Planets,
            "planet" => {
                if args.len() < 2 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("status"),
                        expected: String::from("status planet <planet_id>"),
                    });
                }
                StatusTarget::Planet { id: args[1].to_string() }
            }
            "player" => StatusTarget::Player,
            _ => return Err(CommandError::InvalidArgument {
                command: String::from("status"),
                argument: args[0].to_string(),
                reason: String::from("valid targets are: planets, planet <id>, player"),
            }),
        };

        Ok(StatusArgs { target })
    }
}

pub fn execute(args: StatusArgs, game_state: &GameState) -> Result<CommandEffect, CommandError> {
    let message = match args.target {
        StatusTarget::Turn => format_turn(game_state),
        StatusTarget::Planets => format_planets_list(game_state),
        StatusTarget::Planet { id } => format_planet_detail(&id, game_state)?,
        StatusTarget::Player => format_player_status(game_state),
    };

    Ok(CommandEffect::None { message })
}

fn format_turn(game_state: &GameState) -> String {
    format!("Current turn: {}", game_state.turn)
}

fn format_planets_list(game_state: &GameState) -> String {
    let mut msg = String::from("=== Planets ===\n");
    for planet in game_state.map.planets.values() {
        let owner = match planet.get_owner() {
            Some(id) => id.as_str(),
            None => "uncolonized",
        };
        msg.push_str(&format!("{} ({}) - {}\n", planet.name, planet.id, owner));
    }
    msg
}

fn format_planet_detail(planet_id: &str, game_state: &GameState) -> Result<String, CommandError> {
    let planet = game_state.map.planets.get(planet_id)
        .ok_or_else(|| CommandError::UnknownPlanet(planet_id.to_string()))?;

    let owner = match planet.get_owner() {
        Some(id) => id.clone(),
        None => String::from("uncolonized"),
    };

    let mut msg = format!("=== {} ({}) ===\n", planet.name, planet.id);
    msg.push_str(&format!("Owner: {}\n", owner));
    msg.push_str(&format!("Connections: {:?}\n",
        planet.get_connections().iter().map(|c| &c.to).collect::<Vec<_>>()
    ));
    // TODO: Add resources, structures when visibility allows

    Ok(msg)
}

fn format_player_status(game_state: &GameState) -> String {
    let current_player_id = game_state.current_player();
    let player = game_state.players.get(current_player_id)
        .expect("Current player not found in players map");

    let mut msg = format!("=== {} ===\n", player.name);
    msg.push_str(&format!("Planets owned: {}\n", player.planets.len()));
    for planet_id in &player.planets {
        if let Some(planet) = game_state.map.planets.get(planet_id) {
            msg.push_str(&format!("  - {} ({})\n", planet.name, planet.id));
        }
    }

    msg
}
