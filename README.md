# Colony Protocol

A turn-based, terminal-based space strategy game built in Rust. Conquer planets, manage resources, and build fleets in an epic terminal-based 4X experience.

## Overview

**ColonyProtocol** is a competitive empire-building game where you start with a single planet in a node-graph star system and expand through conquest, resource management, and strategic fleet deployment. Each turn, your colonies produce resources, fleets move along star lanes, and battles resolve when opposing forces meet. An AI opponent races to grow their own empire and challenge your control of the system.

### Core Features

- **Turn-Based Strategy**: Plan your moves, execute them, then watch the turn resolve
- **Resource Management**: Balance minerals, gas, and energy production across your empire
- **Fleet Combat**: Rock-paper-scissors counter system with strategic composition choices
- **Planetary Conquest**: Bombard enemy shields, then colonize with ark ships
- **Building Progression**: Upgrade structures to unlock new capabilities
- **Connected Star Systems**: Node-graph map with travel distances between planets

## Quick Start

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))

### Building and Running

```bash
# Clone the repository
git clone https://github.com/yourusername/ColonyProtocol.git
cd ColonyProtocol

# Build and run
cargo run --release

# Or build separately
cargo build --release
./target/release/colony_protocol
```

### First Game

When you start, you'll see:
```
Initializing command interface...
Type 'help' for available commands

Turn 1 - Player: YourName
>
```

Try these commands to get started:
```bash
status home         # View your starting planet
map                 # See the star system
help               # List all commands
build home mine     # Start building a mine
end                # End your turn
```

## Game Mechanics

### Resources

Three core resources drive your empire:

- **Minerals** 🪨 - Primary building material, produced by mines
- **Gas** 💨 - Advanced technology fuel, produced by refineries
- **Energy** ⚡ - Powers all structures, produced by power plants

Resources are produced each turn by operational structures and capped by storage capacity.

### Structures

Build and upgrade structures on your planets:

- **Planetary Capital** - Foundation of every colony (auto-built)
- **Mine** - Produces minerals
- **Refinery** - Produces gas
- **Power Plant** - Produces energy
- **Shipyard** - Builds ships (level determines available ship types)
- **Defense Shield** - Protects planet from bombardment

Example:
```bash
build planet_id mine           # Build a new mine
build planet_id shipyard       # Upgrade existing shipyard
cancel planet_id               # Cancel pending construction
```

### Ships and Fleets

#### Ship Types

1. **Interceptor**
   - Fast strike fighter
   - Counters: Interceptors, Ravagers
   - Best for: Fleet combat

2. **Ravager**
   - Heavy bomber
   - Best for: Destroying planetary shields

3. **Ark**
   - Colony ship
   - Required for: Claiming planets
   - High cost, no combat value

#### Fleet Management

Ships are organized into named fleets for coordinated operations:

```bash
# Build ships
build-ship planet_id interceptor
build-ship planet_id ravager

# Create fleet
fleet create alpha interceptor_1 interceptor_2

# Add more ships
fleet add alpha ravager_1

# View fleets
fleets
ships

# Disband fleet (ships become standalone)
fleet disband alpha
```

## Command Reference

### Information Commands

```bash
help                    # List all commands
status <planet_id>     # View planet details
map                    # View star system connections
ships                  # List all your ships
fleets                 # List all your fleets
```

### Building Commands

```bash
build <planet_id> <structure_id>        # Build or upgrade structure
build-ship <planet_id> <ship_id>        # Build a ship
cancel <planet_id>                      # Cancel pending action
```

Available structures: `planetary_capital`, `mine`, `refinery`, `power_plant`, `shipyard`, `defense_shield`

Available ships: `interceptor`, `ravager`, `ark`

### Fleet Commands

```bash
fleet create <name> <ship_id> [ship_id...]     # Create new fleet
fleet add <fleet_id> <ship_id> [ship_id...]    # Add ships to fleet
fleet remove <fleet_id> <ship_id> [ship_id...] # Remove ships from fleet
fleet disband <fleet_id>                       # Disband fleet
fleet move <fleet_id> <planet_id>              # Move fleet to planet
fleet bombard <fleet_id>                       # Start bombardment
fleet cancel-bombard <fleet_id>                # Stop bombardment
fleet colonize <fleet_id>                      # Colonize planet (requires Ark)
```

### Running Tests

```bash
cargo test
```

## Roadmap

Future features under consideration:

- [ ] AI opponent improvements
- [ ] Save/load game state
- [ ] Multiple win conditions (score, domination, time limit)
- [ ] Fleet retreat mechanics
- [ ] Fog of war
- [ ] Diplomatic options
- [ ] Planet specialization bonuses

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE) - see the LICENSE file for details.

---

**Ready to conquer the galaxy?** Run `cargo run --release` and build your empire! 🚀
