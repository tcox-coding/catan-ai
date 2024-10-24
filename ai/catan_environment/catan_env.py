from pettingzoo import AECEnv
from websockets.sync.client import connect
from websockets.sync.client import ClientConnection
from gymnasium.spaces import Discrete, MultiDiscrete
import functools
import gymnasium as gym
import json
from numbers import Number
from typing import List

from catan_environment.websocket_interpreter import WebSocketInterpreter

class CatanEnv(gym.Env):
    metadata = {
        "name": "catan_env_v0",
    }

    def __init__(self):
        self._reset_websocket_connection()
        self.action_space = self.set_action_space()
        self.observation_space = self.set_observation_space()
        self.websocket_interpreter = WebSocketInterpreter()
        self._max_episode_timesteps = 1e3
        self._current_episode_timesteps = 0
    
    def _reset_websocket_connection(self):
        self.ws_connection = connect("ws://192.168.1.108:8080/handleMove")

    def step(self, action: List[Number]):
        action_type = ""
        self._current_episode_timesteps += 1

        if self._current_episode_timesteps == self._max_episode_timesteps:
            # If the move isn't valid, then we return that the action was unsuccessful and the observation of the state.
            self._current_episode_timesteps = 0
            observation = self.websocket_interpreter.get_observation()
            reward = 0
            terminated = True
            truncated = False
            info = None

            return (observation, reward, terminated, truncated, info)
            

        if action[0] == 0:
            action_type = "RollDice"
        elif action[0] == 1:
            action_type = "PlaceRobber"
        elif action[0] == 2:
            action_type = "PlaySettlement"
        elif action[0] == 3:
            action_type = "PlayRoad"
        elif action[0] == 4:
            action_type = "PlayCity"
        elif action[0] == 5:
            action_type = "OfferTrade"
        elif action[0] == 6:
            action_type = "AcceptTrade"
        elif action[0] == 7:
            action_type = "DeclineTrade"
        elif action[0] == 8:
            action_type = "PlayDevelopmentCard"
        elif action[0] == 9:
            action_type = "DrawDevelopmentCard"
        elif action[0] == 10:
            action_type = "Discard"
        elif action[0] == 11:
            action_type = "EndTurn"
        else:
            # If the move isn't valid, then we return that the action was unsuccessful and the observation of the state.
            observation = self.websocket_interpreter.get_observation()
            reward = -1
            terminated = json.loads(self.websocket_interpreter.get_json())["game"]["game_ended"]
            truncated = False
            info = None

            return (observation, reward, terminated, truncated, info)


        self.ws_connection.send(json.dumps({
            "command": "take_action",
            "action": {"action_type": action_type, "action_metadata": action[1:]}
        }))
        # Get the previous observation of the game.
        pre_obs = json.loads(self.websocket_interpreter.get_json())

        # Get the observation after the action has been taken.
        self.websocket_interpreter.set_json(self.ws_connection.recv())

        # Set the return values.
        observation = self.websocket_interpreter.get_observation()
        reward = self.__reward(pre_obs, json.loads(self.websocket_interpreter.get_json()))
        terminated = json.loads(self.websocket_interpreter.get_json())["game"]["game_ended"]
        truncated = False
        info = None

        return (observation, reward, terminated, truncated, info)
        

    def reset(self, seed=None, options=None):
        self.ws_connection.send(json.dumps({
            "command": "new_game"
        }))
        self.websocket_interpreter.set_json(self.ws_connection.recv())
        observation = self.websocket_interpreter.get_observation()
        info = None

        return (observation, info)

    def __reward(self, pre_obs, post_obs) -> Number:
        player_id = pre_obs["game"]["current_player_id"]
        pre_player = pre_obs["game"]["players"][player_id]
        post_player = post_obs["game"]["players"][player_id]

        reward = 0
        reward_for_vp = 8

        # Reward for victory points.
        if post_player["victory_points"] > pre_player["victory_points"]:
            reward += reward_for_vp * (post_player["victory_points"] - pre_player["victory_points"])

        # Reward for playing a city (2 * vp).
        if post_player["num_unplaced_cities"] < pre_player["num_unplaced_cities"]:
            reward += reward_for_vp * 2

        # Reward for playing a settlement (2 * vp).
        if post_player["num_unplaced_settlements"] < pre_player["num_unplaced_settlements"]:
            reward += reward_for_vp * 2

        # Reward for placing roads. (1/4 of vp.)
        if post_player["num_unplaced_roads"] < pre_player["num_unplaced_roads"]:
            reward += reward_for_vp / 4

        # Reward for getting a development card. (1/2 of getting vp.)
        if post_player["development_cards_drawn_this_turn"]["RoadBuilding"] > pre_player["development_cards_drawn_this_turn"]["RoadBuilding"]:
            reward += reward_for_vp / 2
        if post_player["development_cards_drawn_this_turn"]["Monopoly"] > pre_player["development_cards_drawn_this_turn"]["Monopoly"]:
            reward += reward_for_vp / 2
        if post_player["development_cards_drawn_this_turn"]["YearOfPlenty"] > pre_player["development_cards_drawn_this_turn"]["YearOfPlenty"]:
            reward += reward_for_vp / 2
        if post_player["development_cards_drawn_this_turn"]["Knight"] > pre_player["development_cards_drawn_this_turn"]["Knight"]:
            reward += reward_for_vp / 2
        if post_player["development_cards_drawn_this_turn"]["VictoryPoint"] > pre_player["development_cards_drawn_this_turn"]["VictoryPoint"]:
            reward += reward_for_vp / 2

        # Reward for getting longest road. (2 * vp)
        if post_player["longest_road"] and not pre_player["longest_road"]:
            reward += reward_for_vp * 2

        # Reward for getting largest army.
        if post_player["largest_army"] and not pre_player["largest_army"]:
            reward += reward_for_vp * 2

        # Punishment for if the action was not successful.
        if not post_obs["last_action_successful"]:
            reward -= reward_for_vp / 8

        return reward

    def render(self):
        pass

    @functools.lru_cache(maxsize=None)
    def set_observation_space(self) -> MultiDiscrete:
        observation_space = []
        # Tiles
        observation_space.extend([
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
            6, 11,
        ])

        # Banks
        observation_space.extend([
            19,
            19,
            19,
            19,
            19,
        ])

        # Development Cards in bank.
        observation_space.extend([25])

        # Player's Resources
        observation_space.extend([20, 20, 20, 20, 20])

        # Player's Development Cards
        observation_space.extend([20, 20, 20, 20, 20])

        # Edges
        observation_space.extend([
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        ])
        
        # Nodes - Settlements
        observation_space.extend([
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        ])

        # Nodes - Cities
        observation_space.extend([
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        ])

        # Ports
        observation_space.extend([
            6, 6, 6, 6, 6, 6, 6, 6, 6
        ])

        # Player's Metadata (vps, knights_played, num_cities_left, num_settlements_left, num_roads_left)
        observation_space.extend([
            11, 15, 5, 6, 16, 2, 2,
            11, 15, 5, 6, 16, 2, 2,
            11, 15, 5, 6, 16, 2, 2,
            11, 15, 5, 6, 16, 2, 2,
        ])

        # Last Dice Roll
        observation_space.extend([
            13,
        ])

        return MultiDiscrete(observation_space)
    
    @functools.lru_cache(maxsize=None)
    def set_action_space(self) -> MultiDiscrete:
        # Action,
        # Possible edge values (2)
        # Possible trade value (8)
        return MultiDiscrete([12, 72, 72, 11, 11, 11, 11, 11, 11, 11, 11])