use clap::Parser;
use rl_agent::{Action, Environment, Robot};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Length of each side of the square grid
    #[arg(short, long, default_value_t = 10)]
    grid_dimensions: usize,

    /// Number of cans to populate the grid with.
    #[arg(short, long, default_value_t = 30)]
    initial_can_count: usize,

    /// Number of episodes
    #[arg(short, long, default_value_t = 5000)]
    n_episodes: usize,

    /// Number of steps in each episode
    #[arg(short, long, default_value_t = 200)]
    m_steps: usize,

    /// Eta
    #[arg(long, default_value_t = 0.2)]
    eta: f32,

    /// Gamma
    #[arg(long, default_value_t = 0.9)]
    gamma: f32,
}

fn main() {
    let args = Args::parse();

    let mut robby = Robot::new();

    for episode in 0..args.n_episodes {
        println!("Episode {episode}:");
        let mut environment =
            Environment::new_randomized(args.grid_dimensions, args.initial_can_count);

        let mut episode_reward: f32 = 0.0;
        let mut episode_actions: Vec<Action> = Vec::new();

        // println!("Initial environment: {:?}", environment);

        for _ in 0..args.m_steps {
            let p = environment.create_percept();
            let a = robby.select_action(&p);
            let reward_amount = environment.calculate_reward(&a);
            episode_reward += reward_amount;
            environment.transition_state(&a);
            let resulting_p = environment.create_percept();
            robby.reward(reward_amount, args.eta, args.gamma, &resulting_p);
            episode_actions.push(a);
        }

        println!("Episode Reward: {}", episode_reward);
        println!("Crashes: {}", environment.crash_count);
        // let actions_string = episode_actions
        //     .iter()
        //     .map(|a| a.to_string())
        //     .collect::<Vec<String>>()
        //     .join("");
        // println!("Actions: {}", actions_string);
    }
}
