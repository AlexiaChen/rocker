extern crate pretty_env_logger;
#[macro_use] extern crate log;

extern crate app;
use app::{App, AppError, Args, Cmd, Opt, OptTypo, OptValue, OptValueParse};
use std::{fmt, path::PathBuf};


#[derive(Debug, Default, Clone)]
pub struct CmdConfig {
    pub enable_tty: bool,
    pub run_command: Vec<PathBuf>,
}

fn main() {
    pretty_env_logger::init();
    info!("such information");

    let mut default = CmdConfig::default();
    let app = CmdConfig::make_app_config(&mut default);
    let helper = app.parse_args();

    default
    .check_and_call(helper.current_cmd_str())
    .map_err(|e| {
        helper.help_cmd_err_exit(helper.current_cmd_ref(), e, 1)
    })
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
            .opt(
                Opt::new("ti", &mut config.enable_tty)
                .long("tty")
                .help("enable tty")
            )
            .args(
                Args::new("command", &mut config.run_command)
                .help("run specific command")
            )
        )
    }

    fn check_and_call(&self, cmd: Option<&str>) -> Result<(), String> {
        println!("Match Cmd: {:?}", cmd);
        match cmd {
            Some("run") => {
               println!("Here is run call");
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}