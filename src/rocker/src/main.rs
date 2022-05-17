extern crate pretty_env_logger;
#[macro_use]
extern crate log;

extern crate app;
use anyhow::{Error, Result};
use app::{App, Args, Cmd, Opt};
use container::Container;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct CmdConfig {
    pub enable_tty: bool,
    pub run_command: Vec<PathBuf>,
    init_command: Vec<PathBuf>,
}

fn main() {
    pretty_env_logger::init();
    info!("hello rocker");

    let mut default = CmdConfig::default();
    let app = CmdConfig::make_app_config(&mut default);
    let helper = app.parse_args();

    default
        .check_and_call(helper.current_cmd_str())
        .map_err(|e| helper.help_cmd_err_exit(helper.current_cmd_ref(), e, 1))
        .unwrap();
}

impl CmdConfig {
    fn make_app_config(config: &mut CmdConfig) -> App {
        App::new("rocker")
        .author("MathxH Chen", "brainfvck@foxmail.com")
        .addr("Github", "https://github.com/AlexiaChen/rocker")
        .desc("A simple container runtime implemented in Rust")
        .version("0.0.1")
        .cmd(
            Cmd::new("run")
            .desc("run container. example: sudo RUST_LOG=trace ./rocker run --tty /bin/sh")
            .opt(
                Opt::new("tty", &mut config.enable_tty)
                .long("tty")
                .short('t')
                .help("enable tty")
            )
            .args(
                Args::new("command", &mut config.run_command)
                .help("run specific command")
            )
        )
        .cmd(
            Cmd::new("init")
            .desc("must be used in internal of rocker. example: rocker init /bin/sh")
            .args(
                Args::new("command", &mut config.init_command)
                .help("init specific command. (WARNING: this command cannot be called from external, it only used in internal)")
            )
        )
    }

    fn check_and_call(&self, cmd: Option<&str>) -> Result<(), String> {
        println!("Match Cmd: {:?}", cmd);
        match cmd {
            Some("run") => {
                run(self.enable_tty, self.run_command[0].to_str().unwrap());
            }
            Some("init") => {
                init(self.init_command[0].to_str().unwrap())
                    .map_err(|e| error!("init failed: {}", e.to_string()))
                    .unwrap();
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

fn run(tty: bool, cmd: &str) {
    debug!("rocker run  tty:{}, cmd:{}", tty, cmd);

    let parent = Container::create_parent_process(tty, cmd);
    if parent.is_err() {
        error!("create parent process failed");
        std::process::exit(-1);
    }

    trace!("waiting parent finish");
    let exit = parent.unwrap().wait().unwrap();
    trace!("parent process wait finished exit status is {}", exit);

    std::process::exit(-1);
}

fn init(cmd: &str) -> Result<()> {
    debug!("rocker init cmd:{}", cmd);
    Container::init_process(cmd, &[])
}
