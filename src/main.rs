use std::{fs::File, io::Write};

use clap::Parser;
use rl_agent::{Action, Environment, Robot};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Length of each side of the square grid
    #[arg(short, long, default_value_t = 10)]
    grid_dimensions: usize,

    /// Number of cans to populate the grid with.
    #[arg(short, long, default_value_t = 50)]
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

struct EpisodeRecord {
    episode_id: usize,
    episode_reward: f32,
    crash_count: usize,
    running_average: f32,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut robby = Robot::new();

    let mut episodes: Vec<EpisodeRecord> = Vec::with_capacity(args.n_episodes);

    for episode_id in 0..args.n_episodes {
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

        let last_few: Vec<f32> = episodes[episode_id.saturating_sub(100)..episode_id]
            .iter()
            .map(|e| e.episode_reward)
            .collect();
        let mut sum: f32 = 0.0;
        for record in &last_few {
            sum += record;
        }

        let running_average = sum / last_few.len() as f32;

        let record = EpisodeRecord {
            episode_id,
            episode_reward,
            crash_count: environment.crash_count,
            running_average,
        };

        episodes.push(record);

        // reduce epsilon every 50 episodes
        if (episode_id + 1) % 50 == 0 {
          robby.epsilon *= 0.99;
        }
    }

    let episode_file_path = "episodes.csv";
    let mut episodes_file = File::create(episode_file_path)?;
    writeln!(
        episodes_file,
        "episode_id,episode_reward,running_avg,crash_count"
    )?;

    let episodes_string = episodes
        .iter()
        .map(|e| {
            format!(
                "{},{},{},{}",
                e.episode_id, e.episode_reward, e.running_average, e.crash_count
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    write!(episodes_file, "{}", episodes_string)?;

    let weight_file_path = "weights.csv";
    let mut weights_file = File::create(weight_file_path)?;
    write!(weights_file, "Current,North,South,East,West")?;
    for i in 0..5 {
        write!(weights_file, ",{}", Action::from(i))?;
    }
    writeln!(weights_file)?;

    let x = robby
        .percept_map
        .iter()
        .map(|(p, i)| {
            format!(
                "{},{},{},{},{},{},{},{},{},{}",
                p.current,
                p.north,
                p.south,
                p.east,
                p.west,
                robby.q_matrix[*i][0],
                robby.q_matrix[*i][1],
                robby.q_matrix[*i][2],
                robby.q_matrix[*i][3],
                robby.q_matrix[*i][4],
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    write!(weights_file, "{}", x)?;

    Ok(())
}
