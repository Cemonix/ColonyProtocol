# ColonyProtocol - Claude Development Guide

## Project Overview

**ColonyProtocol** is a turn-based, terminal-based space strategy game built in Rust. Think Tribal Wars meets Stellaris-lite in your terminal.

### Core Gameplay
- Start with a single planet in a node-graph star system
- Expand by conquering planets, managing resources, upgrading buildings
- Build fleets and engage in simple counter-based combat
- AI opponent races to grow their empire and challenge you
- Each turn: colonies produce resources, fleets move, battles resolve
- Clean text interface focused on decision-making and tactical planning

### Technical Stack
- **Language**: Rust (learning-focused implementation)
- **Interface**: Terminal-based with command parser
- **Architecture**: Turn-based state machine with pending actions queue

## Development Philosophy

### Learning-First Approach
- **Explain, don't just code**: Every implementation should be explained
- **Ask questions**: If design choices aren't clear, discuss options first

### Code Quality
- **Idiomatic Rust**: Prefer Rust patterns
- **Readable over clever**: Clear code beats clever code
- **No unnecessary features**: Only implement what's needed now

### Testing Strategy (TDD-lite)
- **Test important logic**: Command parsing, validation, game state changes
- **Don't test everything**: Skip trivial getters/setters, simple constructors
- **Integration over unit**: Prefer testing full command flows over isolated functions
- **Write tests first for**: Complex parsing logic, game rules, resource calculations
- **Skip tests for**: Basic data structures, simple enums, obvious code

#### When to Write Tests
✅ **Do test:**
- Command parsing (does "planet build c418 mine" parse correctly?)
- Validation logic (does build check for resources?)
- Game state mutations (does building actually update state?)
- Turn processing (do pending actions complete?)
- Combat resolution (do counters work correctly?)

❌ **Don't test:**
- Simple struct constructors
- Basic getters/setters
- Enum definitions
- Type conversions
- Trivial helper functions

## Project Structure

```
colony_core/          # Game engine and logic
colony_cli/          # Terminal interface
colony_protocol/     # Main binary
```

## Command System Architecture

### Design Pattern
Using **macro-based command registration** to reduce boilerplate:
- Commands defined via macro (easy to add new ones)
- Each command implements `CommandExecutor` trait
- Three-phase execution: Parse → Validate → Execute

### Command Flow
1. **Parse**: String → Command struct (syntax check only)
2. **Validate**: Check game state constraints (resources, ownership, etc.)
3. **Execute**: Mutate game state, add pending actions, return response

### Command Format
```
<entity> <action> <target> [options]
planet   build    c418     metal-mine
fleet    move     fleet-1  sector-5
status   planet   c418
```

## Interaction Guidelines for Claude

### DO
- Explain Rust concepts when they come up (`&[&str]`, ownership, traits, etc.)
- Discuss design tradeoffs before implementing
- Show examples of idiomatic Rust patterns
- Point out potential issues (borrowing problems, performance, etc.)
- Suggest when tests would be valuable
- Ask clarifying questions about game design

### DON'T
- Write large chunks of code without explanation
- Add features that weren't requested
- Write tests for trivial code
- Over-engineer solutions
- Skip discussing design choices

### Code Review Focus
When reviewing user's code:
- Is it idiomatic Rust?
- Are there ownership/borrowing issues?
- Is error handling appropriate?
- Are types well-chosen?
- Is it readable and maintainable?

**Remember**: This is a learning project. Understanding > Speed. Ask questions, discuss design, explain concepts.
