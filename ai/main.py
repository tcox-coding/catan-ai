import numpy as np
import torch
import argparse
import os

from ai.replay_buffer import ReplayBuffer
from ai.TD3 import TD3
from catan_environment.catan_env import CatanEnv


# Runs policy for X episodes and returns average reward
# A fixed seed is used for the eval environment
def eval_policy(policy, eval_episodes=10):
    eval_env = CatanEnv()

    avg_reward = 0.
    for _ in range(eval_episodes):
        state, info = eval_env.reset()
        done = False
        while not done:
            action = policy.select_action(np.array(state))
            state, reward, done, _, _ = eval_env.step([round(x) for x in action.tolist()])
            avg_reward += reward

    avg_reward /= eval_episodes

    print("---------------------------------------")
    print(f"Evaluation over {eval_episodes} episodes: {avg_reward:.3f}")
    print("---------------------------------------")
    return avg_reward

def clamp(n, min_value, max_value):
    return max(min_value, min(n, max_value))

def add_noise(action, expl_noise):
    new_action = []
    for i, val in enumerate(action.tolist()):
        if i == 0:
            new_action.append(clamp(val + np.random.normal(0, 11 * expl_noise), 0, 11))
        elif i == 1 or i == 2:
            new_action.append(clamp(val + np.random.normal(0, 72 * expl_noise), 0, 72))
        else:
            new_action.append(clamp(val + np.random.normal(0, 10 * expl_noise), 0, 10))

    return new_action

if __name__ == "__main__":
    torch.backends.cudnn.benchmark = True

    parser = argparse.ArgumentParser()
    parser.add_argument("--start_timesteps", default=100e3, type=int)# Time steps initial random policy is used
    parser.add_argument("--eval_freq", default=5e3, type=int)       # How often (time steps) we evaluate
    parser.add_argument("--max_timesteps", default=1e12, type=int)   # Max time steps to run environment
    parser.add_argument("--expl_noise", default=0.1, type=float)    # Std of Gaussian exploration noise
    parser.add_argument("--batch_size", default=1024, type=int)      # Batch size for both actor and critic
    parser.add_argument("--discount", default=0.99, type=float)     # Discount factor
    parser.add_argument("--tau", default=0.005, type=float)         # Target network update rate
    parser.add_argument("--policy_noise", default=0.2)              # Noise added to target policy during critic update
    parser.add_argument("--noise_clip", default=0.5)                # Range to clip target policy noise
    parser.add_argument("--policy_freq", default=2, type=int)       # Frequency of delayed policy updates
    parser.add_argument("--save_model", default= True, action="store_true")        # Save model and optimizer parameters
    parser.add_argument("--load_model", default="")                 # Model load file name, "" doesn't load, "default" uses file_name
    args = parser.parse_args()

    file_name = f"CatanAI"
    print("---------------------------------------")
    print(f"Env: CatanAI")
    print("---------------------------------------")

    if not os.path.exists("./results"):
        os.makedirs("./results")

    if args.save_model and not os.path.exists("./models"):
        os.makedirs("./models")

    env = CatanEnv()

    kwargs = {
        "state_dim": env.observation_space.shape[0],
        "action_dim": env.action_space.shape[0],
        "max_action": 72,
        "discount": 0.99,
        "tau": 0.005,
        "policy_noise": 0.2 * 72,
        "noise_clip": 0.5 * 72,
        "policy_freq": 2
    }

    policy = TD3(**kwargs)

    if args.load_model != "":
        policy_file = file_name if args.load_model == "default" else args.load_model
        policy.load(f"./models/{policy_file}")

    replay_buffer = ReplayBuffer(kwargs["state_dim"], kwargs["action_dim"])
    
    # Evaluate untrained policy
    # evaluations = [eval_policy(policy)]

    state, info = env.reset()
    done = False
    episode_reward = 0
    episode_timesteps = 0
    episode_num = 0

    # Max Timesteps
    for t in range(int(args.max_timesteps)):
        episode_timesteps += 1

        if t < args.start_timesteps:
            # Select action randomly
            action = env.action_space.sample()
        else:
            # Select action according to policy.
            action = np.array(add_noise(policy.select_action(np.array(state)), args.expl_noise))

        # Perform action
        next_state, reward, done, _, _ = env.step([round(x) for x in action.tolist()]) 
        done_bool = float(done) if episode_timesteps < env._max_episode_timesteps else 0

        # Store data in replay buffer
        replay_buffer.add(state, action, next_state, reward, done_bool)

        state = next_state
        episode_reward += reward

        # Train agent after collecting sufficient data
        if t >= args.start_timesteps:
            policy.train(replay_buffer, args.batch_size)

        if done: 
            # +1 to account for 0 indexing. +0 on ep_timesteps since it will increment +1 even if done=True
            print(f"Total T: {t+1} Episode Num: {episode_num+1} Episode T: {episode_timesteps} Reward: {episode_reward:.3f}")
            # Reset environment
            state, info = env.reset()
            done = False
            episode_reward = 0
            episode_timesteps = 0
            episode_num += 1 

        # Evaluate episode
        if (t + 1) % args.eval_freq == 0:
            # evaluations.append(eval_policy(policy))
            # np.save(f"./results/{file_name}", evaluations)
            if args.save_model: policy.save(f"./models/{file_name}")