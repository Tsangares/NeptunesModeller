# Game Info CLI

This script provides a command-line interface to fetch information about open games in Neptune's Pride. The game information can be retrieved for three game types: 64-player games, Proteus games, and new player games. You can obtain the game ID, name, or raw game information.

## Functions

The script contains the following functions:

1. `get_open_games()` - Returns a list of open games.
2. `get_game_by_type(game_type: str, title: str = None)` - Returns a game of a specific type.
3. `get_64p_game()` - Returns a 64-player game.
4. `get_proteus_game()` - Returns a Proteus game.
5. `get_noob_game()` - Returns a new player game.
6. `get_game_id(game: GameInfo)` - Returns the game ID.
7. `get_game_name(game: GameInfo)` - Returns the game name.
8. `get_latest_64p_game_id()` - Returns the latest 64-player game ID.
9. `get_latest_proteus_game_id()` - Returns the latest Proteus game ID.
10. `get_lastest_noob_game_id()` - Returns the latest new player game ID.
11. `get_latest_64p_game_name()` - Returns the latest 64-player game name.
12. `get_latest_proteus_game_name()` - Returns the latest Proteus game name.
13. `get_lastest_noob_game_name()` - Returns the latest new player game name.

## Usage

```bash
python game_info.py <game_type> [<info_type>] [--test]
```

### Arguments

- `game_type`: Type of game to get information for. Choices: `big_game`, `proteus`, `noob`.
- `info_type`: (Optional) Type of information to get. Choices: `id`, `name`, `raw`. If not specified, it will print both the game name and ID.
- `--test`: (Optional) Run unit tests.

## Example

```bash
python game_info.py big_game id
```
This will print the latest 64-player game ID.
