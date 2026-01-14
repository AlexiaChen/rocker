// mathxh@MathxH:~/Project/rocker/target/debug$ sudo ./cg upper
// forked process PID 1479
// waiting child process PID 1479
// enrty CURRENT_PROC pid 1
// ^C
// mathxh@MathxH:~/Project/rocker/target/debug$ sudo ./cg lower
// forked process PID 1485
// waiting child process PID 1485
// enrty CURRENT_PROC pid 1
// stress: FAIL: [3] (415) <-- worker 4 got signal 9
// stress: WARN: [3] (417) now reaping child worker processes
// stress: FAIL: [3] (421) kill error: No such process
// stress: FAIL: [3] (451) failed run completed in 0s
// /bin/sh: 1: stress:: not found
// leave CURRENT_PROC pid 1
// wait finished child process PID 1485
// mathxh@MathxH:~/Project/rocker/target/debug$

// The result is that a low memory limit (100M) automatically kills the stress process with 200M of memory,
// and a high memory limit (300M) keeps the 200M stress process running.

use cgroups_rs::cgroup_builder::*;
use cgroups_rs::*;
use std::{env, process};
use std::{thread, time};
use unshare::{Command, GidMap, Namespace, UidMap};

const NO_ROOT_PRV: u32 = 1;
const CURRENT_PROC: &str = "/proc/self/exe";

const UPPER_MEM_LIMIT: i32 = 300 * 1024 * 1024;
const LOWER_MEM_LIMIT: i32 = 100 * 1024 * 1024;

fn main() {
    let arg0 = env::args().next().unwrap();
    // chekc if it self in container
    if arg0 == CURRENT_PROC {
        // wait for cgroup build and add_task finished, then go run stress
        thread::sleep(time::Duration::from_secs(3));
        println!("enrty CURRENT_PROC pid {}", process::id());
        // sudo apt install stress
        let cmd_result = Command::new("/bin/sh")
            .arg("-c")
            .arg("`stress --vm-bytes 200m --vm-keep -m 1`")
            .status();

        if cmd_result.is_err() {
            println!(
                "container process error is: {}",
                cmd_result.err().unwrap()
            );
            process::exit(1);
        }
        println!("leave CURRENT_PROC pid {}", process::id());
        return;
    }

    if env::args().len() != 2 {
        println!("usage: ./cg upper (upper memory limit) or ./cg lower (lower memory limit)");
        return;
    }

    let cmd_result = Command::new(CURRENT_PROC)
        .unshare(&[Namespace::Uts, Namespace::Pid, Namespace::Mount])
        .set_id_maps(
            vec![UidMap {
                inside_uid: NO_ROOT_PRV,
                outside_uid: NO_ROOT_PRV,
                count: 1,
            }],
            vec![GidMap {
                inside_gid: NO_ROOT_PRV,
                outside_gid: NO_ROOT_PRV,
                count: 1,
            }],
        )
        .spawn();

    if cmd_result.is_err() {
        println!("Spawn ERROR {}", cmd_result.err().unwrap());
        process::exit(1)
    } else {
        let mut child = cmd_result.unwrap();
        println!("forked process PID {}", child.pid());

        let mut mem_limit = UPPER_MEM_LIMIT;
        let arg1 = env::args().nth(1).unwrap();
        if arg1 == "upper" {
            mem_limit = UPPER_MEM_LIMIT;
        } else if arg1 == "lower" {
            mem_limit = LOWER_MEM_LIMIT;
        }

        let h = cgroups_rs::hierarchies::auto();
        let cg: Cgroup = CgroupBuilder::new("hello")
            .memory()
            .memory_hard_limit(mem_limit.into())
            .done()
            .build(h);

        let mem: &cgroups_rs::memory::MemController =
            cg.controller_of().unwrap();
        let res = mem.add_task(&CgroupPid::from(child.pid() as u64));
        if res.is_err() {
            println!("mem addtask failed {}", res.err().unwrap());
            process::exit(1);
        }

        println!("waiting child process PID {}", child.pid());
        let res = child.wait();
        if res.is_err() {
            println!("child process wait failed {}", res.err().unwrap());
            process::exit(1);
        }
        println!("wait finished child process PID {}", child.pid());

        let res = cg.delete();
        if res.is_err() {
            println!("cgroup delete failed {}", res.err().unwrap());
            process::exit(1);
        }
    }
}
