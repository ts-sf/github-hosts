use chrono::{TimeDelta, Utc};
use clap::{Parser, ValueEnum};
use github_hosts::{
    consts::GITHUB_URLS, executor::Executor, util::is_valid_domain, version::VERSION,
};
use std::path::PathBuf;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Target {
    Github,
    File,
}

#[derive(Debug, Parser)]
#[command(name = "hosts")]
#[command(about = "A fictional CLI for hosts", long_about = None, version = VERSION)]
struct Cli {
    #[arg(long)]
    target: Target,

    #[arg(long, default_value_t = false)]
    save: bool,

    file: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let env = EnvFilter::from_env("RUST_LOG");
    tracing_subscriber::registry()
        .with(env)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Cli { target, save, file } = Cli::parse();

    #[allow(unused_assignments)]
    let mut targets = vec![];

    match target {
        Target::Github => targets = GITHUB_URLS.iter().map(|x| x.to_string()).collect(),
        Target::File => {
            let txt = std::fs::read_to_string(file.expect("incorrect file path"))
                .expect("read file error");
            let txt = txt.replace('\r', "\n");

            targets = txt
                .split('\n')
                .filter_map(|line| {
                    let no_white = line.trim();
                    is_valid_domain(no_white).then_some(no_white.to_owned())
                })
                .collect::<Vec<_>>();
        }
    };

    let mut exec = Executor::new();
    exec.resolve(targets.iter()).await;
    exec.fresh_delays().await;

    let answer = exec.best_answer();

    let mut f = String::new();
    answer
        .into_iter()
        .for_each(|(domain, (ip, _))| f.push_str(&format!("{}\t\t{}\n", ip, domain)));

    let dt = Utc::now().checked_add_signed(TimeDelta::hours(8)).unwrap();
    let dt = dt.format("%Y-%m-%d %H:%M:%S").to_string();

    let result = TEMP.replace("{}", &f).replace("{time}", &dt);
    println!("{}", result);

    if save {
        std::fs::write("hosts", result).expect("save error.");
    }
}

const TEMP: &str = r#"
# start

{}

# github repository: https://github.com/ts-sf/github-hosts

# update at {time}

# end 
"#;
