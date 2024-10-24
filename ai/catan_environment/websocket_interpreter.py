from typing import List
from numbers import Number
import json

class WebSocketInterpreter:
    def __init__(self, json_str: str | None = None):
        if json is not None:
            self.json_str = json_str
    
    def set_json(self, json_str: str):
        self.json_str = json_str

    def get_json(self) -> str:
        return self.json_str

    def get_observation(self) -> List[Number]:
        game_json = json.loads(self.json_str)["game"]
        observation = []

        # Tiles
        for tile in game_json["board"]["tiles"]:
            "Forest", "Hills", "Mountains", "Fields", "Plains", "Desert"
            match tile["terrain"]:
                case "Desert":
                    observation.append(0)
                case "Forest":
                    observation.append(1)
                case "Hills":
                    observation.append(2)
                case "Fields":
                    observation.append(3)
                case "Plains":
                    observation.append(4)
                case "Mountains":
                    observation.append(5)
            observation.append(tile["chit"])
        
        # Banks
        observation.append(game_json["bank"]["resource_cards"]["Lumber"])
        observation.append(game_json["bank"]["resource_cards"]["Ore"])
        observation.append(game_json["bank"]["resource_cards"]["Brick"])
        observation.append(game_json["bank"]["resource_cards"]["Wheat"])
        observation.append(game_json["bank"]["resource_cards"]["Sheep"])

        # Development Cards in bank.
        observation.append(len(game_json["bank"]["development_cards"]))

        # Player Resources
        current_player = game_json["players"][game_json["current_player_id"]]
        observation.append(current_player["resource_cards"]["Lumber"])
        observation.append(current_player["resource_cards"]["Ore"])
        observation.append(current_player["resource_cards"]["Brick"])
        observation.append(current_player["resource_cards"]["Wheat"])
        observation.append(current_player["resource_cards"]["Sheep"])

        # Player Development Cards
        observation.append(current_player["development_cards"]["YearOfPlenty"])
        observation.append(current_player["development_cards"]["Knight"])
        observation.append(current_player["development_cards"]["RoadBuilding"])
        observation.append(current_player["development_cards"]["Monopoly"])
        observation.append(current_player["development_cards"]["VictoryPoint"])

        # Edges
        for edge in game_json["board"]["edges"]:
            if edge["building"] != None:
                observation.append(edge["building"]["Road"][1])
            else:
                observation.append(0)

        # Nodes - Settlements
        for node in game_json["board"]["nodes"]:
            if node["building"] != None and "City" in node["building"]:
                observation.append(0)
            elif node["building"] != None:
                observation.append(node["building"]["Settlement"][1])
            else:
                observation.append(0)

        # Nodes - Cities
        for node in game_json["board"]["nodes"]:
            if node["building"] != None and "Settlement" in node["building"]:
                observation.append(0)
            elif node["building"] != None:
                observation.append(node["building"]["City"][1])
            else:
                observation.append(0)

        # Ports
        for port in game_json["board"]["ports"]:
            if port == "Lumber":
                observation.append(0)
            elif port == "Ore":
                observation.append(1)
            elif port == "Brick":
                observation.append(2)
            elif port == "Wheat":
                observation.append(3)
            elif port == "Sheep":
                observation.append(4)
            elif port == "ThreeToOne":
                observation.append(5)

        # Player Metadata
        for player in game_json["players"]:
            # (vps, knights_played, num_cities_left, num_settlements_left, num_roads_left, longest road, largest army)
            observation.append(player["victory_points"])
            observation.append(player["num_knights_played"])
            observation.append(player["num_unplaced_cities"])
            observation.append(player["num_unplaced_settlements"])
            observation.append(player["num_unplaced_roads"])
            observation.append(1 if player["longest_road"] else 0)
            observation.append(1 if player["largest_army"] else 0)
        
        # Last Dice Roll
        observation.append(game_json["previous_dice_roll"])

        return observation
