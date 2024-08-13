# Maze Game

This is a maze game written in Rust that features both 2D and 3D views. Navigate through various levels of mazes while avoiding periodic jumpscares with accompanying sound effects. The game ends with a congratulatory screen upon completion.

## Features

- **2D and 3D Views**: Switch between 2D and 3D perspectives to navigate the maze.
- **Jumpscare Mechanic**: Enemies appear periodically with a sound effect to surprise the player.
- **Background Music**: Play background music during the game.
- **FPS Display**: The current frame rate is displayed in the game.
- **Multiple Levels**: Choose from different maze levels to play.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/): Ensure you have Rust installed on your system. Follow the instructions on the [official website](https://www.rust-lang.org/tools/install).

### Installation

1. Clone the repository:

    ```bash
    git clone <repository_url>
    ```

2. Navigate into the project directory:

    ```bash
    cd <project_directory>
    ```

3. Install dependencies:

    ```bash
    cargo build
    ```

### Running the Game

1. Run the game:

    ```bash
    cargo run
    ```

2. Follow the on-screen instructions to choose a maze level and start the game.

### Controls

- **1, 2, 3**: Select the maze level.
- **M**: Toggle between 2D and 3D views.
- **Esc**: Exit the game.
- **Enter**: Proceed or close screens.

### Audio Files

Ensure the following audio files are present in the project directory:

- `screamer.mp3`: The sound effect played when the enemy appears.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The game uses the `minifb`, `rodio`, `rusttype`, and other crates for rendering, sound, and text.
- Special thanks to the Rust community for their support and contributions.

