extern crate pretty_env_logger;
#[macro_use]
extern crate log;

extern crate app;
use anyhow::Result;
use app::{App, Args, Cmd, Opt};
use cgroups::cgroup_manager::CgroupManager;
use cgroups::subsystems::subsystem::ResourceConfig;
use container::Container;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct CmdConfig {
    pub enable_tty: bool,
    pub memory_limit: Option<String>,
    pub cpu_share: Option<String>,
    pub cpu_set: Option<String>,
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
            .desc("run container. example:\n 
            sudo RUST_LOG=trace ./rocker run --tty /bin/sh \n 
            sudo RUST_LOG=trace ./rocker run --tty \"ls -l\"\n
            sudo RUST_LOG=trace ./rocker run --tty bash")
            .opt(
                Opt::new("tty", &mut config.enable_tty)
                .long("tty")
                .short('t')
                .help("enable tty")
            )
            .opt(Opt::new("memory_limit", &mut config.memory_limit)
                .optional()
                .long("memory")
                .short('m')
                .help("memory limit (default 1024m)")
            )
            .opt(
                Opt::new("cpu_share", &mut config.cpu_share)
                .optional()
                .long("cpushare")
                .help("cpu time weight limit (default 1024)")
            )
            .opt(Opt::new("cpu_set", &mut config.cpu_set)
                .optional()
                .long("cpuset")
                .help("cpu cores limit (default 1-2)")
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
        debug!("Match Cmd: {}", cmd.unwrap());
        match cmd {
            Some("run") => {
                let res = ResourceConfig {
                    memory_limit: Some(
                        self.memory_limit
                            .clone()
                            .unwrap_or(String::from("1024m")),
                    ),
                    cpu_set: Some(
                        self.cpu_share.clone().unwrap_or(String::from("1-2")),
                    ),
                    cpu_shares: Some(
                        self.cpu_set.clone().unwrap_or(String::from("1024")),
                    ),
                };
                run(
                    self.enable_tty,
                    self.run_command[0].to_str().unwrap(),
                    &res,
                );
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

fn run(tty: bool, cmd: &str, res: &ResourceConfig) {
    debug!("rocker run  tty:{}, cmd:{}", tty, cmd);

    let mut parent = Container::create_parent_process(tty, cmd);
    if parent.0.as_ref().is_err() {
        error!("create parent process failed");
        std::process::exit(-1);
    }

    let cgroup_manager = CgroupManager::new("rocker-cgroup");
    cgroup_manager.set(res).unwrap();
    cgroup_manager
        .apply(parent.0.as_ref().unwrap().pid())
        .unwrap();

    // let write_pipe_fd = parent.1;
    // let mut write_pipe = parent.0.as_mut().unwrap().take_pipe_writer(write_pipe_fd).unwrap();
    // write_pipe.write_all(cmd.as_bytes()).unwrap();

    trace!("waiting parent finish");
    let exit = parent.0.as_mut().unwrap().wait().unwrap();
    trace!("parent process wait finished exit status is {}", exit);

    let pwd = std::env::current_dir();
    let pwd = pwd.unwrap().join("busybox");
    let old_root = pwd.join(".pivot_root");
    std::fs::remove_dir_all(old_root.as_os_str().to_str().unwrap())
        .expect("remove old root dir");

    cgroup_manager.destroy().unwrap();

    std::process::exit(-1);
}

fn init(cmd: &str) -> Result<()> {
    debug!("rocker init cmd:{}", cmd);
    Container::init_process(cmd, &[])
}
