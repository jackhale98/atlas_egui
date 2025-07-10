# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Atlas is a CLI application built with Rust for mechanical engineering tolerance stack-up analysis. It manages projects containing components with dimensional features and their relationships (mates), then performs statistical analysis on dimensional chains using an interactive command-line interface.

## Development Commands

- **Build the project**: `cargo build`
- **Run the application**: `cargo run`
- **Run tests**: `cargo test`
- **Check for errors**: `cargo check`
- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`

## Quick Start

1. **Interactive Mode**: `cargo run -- interactive` - Full menu-driven interface
2. **Create Project**: `cargo run -- new "My Project"` - Create a new project
3. **Help**: `cargo run -- --help` - Show all available commands

## CLI Usage

### Basic Commands
- **Create new project**: `atlas new [name]`
- **Open existing project**: `atlas open <path>`
- **Interactive mode**: `atlas interactive`

### Component Management
- **Add component**: `atlas component add`
- **List components**: `atlas component list`
- **Edit component**: `atlas component edit`
- **Remove component**: `atlas component remove`

### Feature Management
- **Add feature**: `atlas feature add`
- **List features**: `atlas feature list`
- **Edit feature**: `atlas feature edit`
- **Remove feature**: `atlas feature remove`

### Analysis
- **Create analysis**: `atlas analysis new`
- **Run analysis**: `atlas analysis run`
- **View results**: `atlas analysis results`
- **Export results**: `atlas analysis export`

### Visualization
- **Show dependencies**: `atlas visualize dependencies`
- **Show results**: `atlas visualize results`
- **Export to SVG**: `atlas visualize export`

## Architecture

### Core Structure
- **`src/main.rs`**: Entry point that parses CLI commands and routes to handlers
- **`src/cli/`**: Command-line interface modules for different commands
- **`src/prompts/`**: Interactive prompts using inquire for user input
- **`src/state/mod.rs`**: Central application state (`AppState`) containing all project data

### Key Modules
- **`config/`**: Data models for projects, components, features, and mates (serialized as RON files)
- **`file/`**: File I/O management for loading/saving project files
- **`cli/`**: Command handlers for different CLI operations
- **`prompts/`**: Interactive prompts with fuzzy search and validation
- **`analysis/`**: Statistical analysis engine for tolerance stackups
- **`visualization/`**: ASCII and SVG output for charts and matrices

### Interactive Features
- **Fuzzy search**: Quick selection of components and features
- **Smart prompts**: Context-aware input validation
- **Progress indicators**: Visual feedback for long-running analyses
- **Rich output**: Colored terminal output with tables and charts

### Data Flow
1. Projects are stored as RON files referencing component and analysis files
2. Components contain dimensional features with tolerances
3. Mates define how features relate to each other (clearance, interference, etc.)
4. Analysis configurations specify which features contribute to dimensional chains
5. Results are computed using Monte Carlo or other statistical methods
6. Visualizations can be displayed as ASCII art or exported as SVG

### Key Dependencies
- **clap**: Command-line argument parsing
- **inquire**: Interactive prompts with fuzzy search
- **console**: Terminal styling and colors
- **textplots**: ASCII chart generation
- **svg**: SVG export capabilities
- **petgraph**: Dependency graph analysis
- **ron**: Rusty Object Notation for serialization
- **serde**: Serialization framework
- **anyhow**: Error handling

## File Structure Patterns
- Project files use `.ron` extension
- Components stored in `components/` subdirectory
- Analyses stored in `analyses/stackups/[uuid]/` subdirectories
- Each analysis has its own UUID-based folder containing `analysis.ron` and results