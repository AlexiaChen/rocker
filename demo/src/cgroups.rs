use unshare::{Command, Namespace, UidMap, GidMap};
use std::{env, process};
use cgroups_rs::*;
use cgroups_rs::cgroup_builder::*;

const NO_ROOT_PRV :u32 = 1;
const CURRENT_PROC :&str= "/proc/self/exe";

fn main() {

    let arg0 = env::args().nth(0).unwrap();
    // chekc if it self in container
    if arg0 == CURRENT_PROC {
        println!("current pid {}", process::id());
        // sudo apt install stress
        let cmd_result = Command::new("/bin/sh")
        .arg("-c")
        .arg0("stress --vm-bytes 200m --vm-keep -m 1")
        .status();
    
        if cmd_result.is_err() {
            println!("container process error is: {}", cmd_result.err().unwrap());
            process::exit(1);   
        }
    }

    let cmd_result = Command::new(CURRENT_PROC)
    .unshare(&[Namespace::Uts, Namespace::Pid, Namespace::Mount])
    .set_id_maps(vec![UidMap{inside_uid:NO_ROOT_PRV, outside_uid:NO_ROOT_PRV, count: 1}], vec![GidMap{inside_gid:NO_ROOT_PRV, outside_gid:NO_ROOT_PRV, count: 1}])
    .spawn();
    
    if cmd_result.is_err() {
        println!("Spawn ERROR {}", cmd_result.err().unwrap());
        process::exit(1)
    } else {
        let mut child = cmd_result.unwrap();
        println!("forked process PID {}", child.pid());

        let h = cgroups_rs::hierarchies::auto();
        let cg: Cgroup = CgroupBuilder::new("hello")
            .memory()
                .memory_hard_limit(100 * 1024 * 1024)
            .done()
            .build(h);

        let mem: &cgroups_rs::memory::MemController = cg.controller_of().unwrap();
        let res = mem.add_task(&CgroupPid::from(child.pid() as u64));
        if res.is_err() {
            println!("mem addtask failed {}", res.err().unwrap());
            process::exit(1);
        }

        let res = child.wait();
        if res.is_err() {
            println!("child process wait failed {}", res.err().unwrap());
            process::exit(1);
        }
        
        let res = cg.delete();
        if res.is_err() {
            println!("cgroup delete failed {}", res.err().unwrap());
            process::exit(1);
        }
    }
}
