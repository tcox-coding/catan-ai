from pettingzoo import AECEnv
from websockets.sync.client import connect
from websockets.sync.client import ClientConnection
from gymnasium.spaces import Discrete, MultiDiscrete
import functools


class CatanEnv(AECEnv):

    metadata = {
        "name": "catan_env_v0",
    }

    def __init__(self):
        self._reset_websocket_connection()
        self.possible_agents = [0, 1, 2, 3]
    
    def _reset_websocket_connection(self):
        self.ws_connection = connect("ws://192.168.1.108:8080/handleMove")


    def __reset__(self, seed=None, options=None):
        pass

    def step(self, actions):
        pass

    def render(self):
        pass

    @functools.lru_cache(maxsize=None)
    def observation_space(self):
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

        # Player's Metadata (vps, knights_played)
        observation_space.extend([
            11, 15,
            11, 15,
            11, 15,
            11, 15,
        ])

        # Last Dice Roll
        observation_space.extend([
            13,
        ])

        return MultiDiscrete(observation_space)
    
    @functools.lru_cache(maxsize=None)
    def action_space(self):
        # Action,
        # Possible edge values (2)
        # Possible trade value (8)
        return MultiDiscrete([12, 72, 72, 11, 11, 11, 11, 11, 11, 11, 11])