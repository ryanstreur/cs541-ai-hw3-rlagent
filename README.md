# CS 541 Programming Assignment 3: Reinforcement Learning

## Getting Started

This program is primarily written in Rust, with a jupyter notebook used to visualize the results. The rust program implements a reinforcement learning agent which simulates a robot on a grid picking up cans. Running the program will produce two csv files: episodes.csv and weights.csv. episodes.csv represents the training epochs, and weights.csv represents the weights for each percept/action combination at the end of the run.

To build the rust program:

1. Make sure you have Rust and Cargo installed
2. In the project root directory, run `cargo build --release`

You should now be able to run the program binary by running `./target/release/rl_agent`

The program has the following command line interface:

```
Usage: rl_agent [OPTIONS]

Options:
  -g, --grid-dimensions <GRID_DIMENSIONS>      Length of each side of the square grid [default: 10]
  -i, --initial-can-count <INITIAL_CAN_COUNT>  Number of cans to populate the grid with [default: 50]
  -n, --n-episodes <N_EPISODES>                Number of episodes [default: 5000]
  -m, --m-steps <M_STEPS>                      Number of steps in each episode [default: 200]
      --eta <ETA>                              Eta [default: 0.2]
      --gamma <GAMMA>                          Gamma [default: 0.9]
  -h, --help                                   Print help
  -V, --version                                Print version
```

## Producing The Visualization

To visualize the csv data, you can 

1. Create a python virtual environment using the environment management mechanism of your choice
2. Install the dependencies from requirements.txt
3. Open the writeup.ipynb file as a jupyter notebook, and run the one cell therein.
