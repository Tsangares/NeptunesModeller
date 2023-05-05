
#%%
import requests, logging, re, unittest
from typing import TypedDict

#Ref: https://docs.python.org/3/library/typing.html#typing.TypedDict
class GameInfo(TypedDict, total=False):
    status: str
    name: str
    creator: str
    turn_based: int
    number: str
    maxPlayers: int
    players: int
    version: "triton"
    game_type: str | None
    
    
def get_open_games() -> list[GameInfo]:
    #Get open games
    data = {'type': 'open_games'}
    response = requests.post('https://np.ironhelmet.com/mrequest/open_games', data=data)
    
    #Handle failure
    if response.status_code!=200:
        raise Exception("Failed to retrieve open games")

    #Parse response to list of games
    raw_games = response.json()[1]
    
    all_games = []
    for game_type,games in raw_games.items():
        all_games += [(g | {"game_type": game_type}) for g in games]
    return all_games

##Get Specific Game Handlers

#Makes removes plural words from sentence
def non_plural(text):
    return re.sub(r'\b\w+s\b', lambda match: match.group()[:-1], text)

#Get a game that should only have one like 64p and proteus
def get_game_by_type(game_type,title=None):
    if title is None: title=game_type
    games = get_open_games()
    game = [g for g in games if g['game_type']==game_type]
    if len(game)==1:
        #Good only one game
        return game[0]
    elif len(game)>1:
        #Multiple games; this function is not for that
        logging.warning(f"Multiple {title}!")
        return game
    else:
        #No games for some reason
        logging.error(f"No {title} available.")
        return []

def get_64p_game() -> GameInfo:
    title="64 Player Games"
    return get_game_by_type('experimental_games',title)

def get_proteus_game() -> GameInfo:
    title = "Official Proteus Games"
    return get_game_by_type('proteus_test_games',title)

def get_noob_game() -> GameInfo:
    title = "New Player Games"
    return get_game_by_type('new_player_games',title)

## Helpers so you don't have to read the dictionary
def get_game_id(game: GameInfo) -> str:
    if isinstance(game, list):
        raise Exception("There are more than one game!")
    return game['number']

def get_game_name(game: GameInfo) -> str:
    if isinstance(game, list):
        raise Exception("There are more than one game!")
    return game['name']

## Quick Functions to Get Latest Game Name/Id 

def get_latest_64p_game_id() -> str:
    game = get_64p_game()
    return get_game_id(game)

def get_latest_proteus_game_id() -> str:
    game = get_proteus_game()
    return get_game_id(game)

def get_lastest_noob_game_id() -> str:
    noob = get_noob_game()
    return get_game_id(noob)

def get_latest_64p_game_name() -> str:
    game = get_64p_game()
    return get_game_name(game)(game)

def get_latest_proteus_game_name() -> str:
    game = get_proteus_game()
    return get_game_id(game)

def get_lastest_noob_game_name() -> str:
    noob = get_noob_game()
    return get_game_name(noob)


#Unit test
class TestGameFunctions(unittest.TestCase):
    def test_get_latest_64p_game_id(self):
        self.assertIsInstance(get_latest_64p_game_id(), str)
    def test_get_latest_proteus_game_name(self):
        self.assertIsInstance(get_latest_proteus_game_name(), str)
    def test_get_lastest_noob_game_name(self):
        self.assertIsInstance(get_lastest_noob_game_name(), str)

if __name__=="__main__":
    import argparse
    # Define argument parser
    parser = argparse.ArgumentParser(description='Process game information.')
    parser.add_argument('game_type', choices=['big_game', 'proteus', 'noob'], help='Type of game to get information for.')
    parser.add_argument('info_type', choices=['id', 'name', 'raw'], nargs='?', default=None, help='Type of information to get.')
    parser.add_argument('--test', action='store_true', help='Run unit tests.')

    # Parse arguments
    args = parser.parse_args()
    
    # Handle game type and info type
    if args.game_type == 'big_game':
        if args.info_type is None:
            print(get_latest_64p_game_name(), get_latest_64p_game_id())
        elif args.info_type == 'id':
            print(get_latest_64p_game_id())
        elif args.info_type == 'name':
            print(get_latest_64p_game_name())
        elif args.info_type == 'raw':
            print(get_64p_game())
    elif args.game_type == 'proteus':
        if args.info_type is None:
            print(get_latest_proteus_game_name(), get_latest_proteus_game_id())
        elif args.info_type == 'id':
            print(get_latest_proteus_game_id())
        elif args.info_type == 'name':
            print(get_latest_proteus_game_name())
        elif args.info_type == 'raw':
            print(get_proteus_game())
    elif args.game_type == 'noob':
        if args.info_type is None:
            print(get_lastest_noob_game_name(), get_lastest_noob_game_id())
        elif args.info_type == 'id':
            print(get_lastest_noob_game_id())
        elif args.info_type == 'name':
            print(get_lastest_noob_game_name())
        elif args.info_type == 'raw':
            print(get_noob_game())

    # Run unit tests
    if args.test:
        unittest.main(argv=[''], exit=False)