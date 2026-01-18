use std::io::{Write, stdin, stdout};

pub fn get_player_input<F, T>(parser: F) -> T
where 
    F: Fn(&str) -> Result<T, String>
{
    loop {
        print!("> ");
        stdout().flush().expect("Failed to flush terminal");
        
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                match parser(input.trim()) {
                    Ok(parsed) => break parsed,
                    Err(error) => {
                        eprintln!("ERROR: {}", error);
                    }
                }
            }
            Err(error) => {
                eprintln!("ERROR: Terminal input failure - {}.", error);
            }
        }
    }
}

/// Convert name to ID (lowercase with underscores)
/// Example: "Crimson Theta" -> "crimson_theta"
pub fn name_to_id(name: &str) -> String {
    name.to_lowercase().replace(' ', "_")
}