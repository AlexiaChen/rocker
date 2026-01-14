use std::{env, process};
use unshare::{Command, GidMap, Namespace, UidMap};

const ROOT_PRV: u32 = 0;
const NO_ROOT_PRV: u32 = 1;

fn main() {
    if env::args().len() != 2 {
        println!(
            "usage: ./ns 0(root user in container) or ./ns 1(no root user in container)"
        );
        return;
    }

    let mut enable_root = false;
    let arg1 = env::args().nth(1).unwrap();
    if arg1 == "0" {
        enable_root = true;
    }

    if enable_root {
        let cmd_result = Command::new("/bin/sh")
            .unshare(&[
                Namespace::Uts,
                Namespace::Ipc,
                Namespace::Pid,
                Namespace::Mount,
                Namespace::User,
                Namespace::Net,
            ])
            .set_id_maps(
                vec![UidMap {
                    inside_uid: ROOT_PRV,
                    outside_uid: ROOT_PRV,
                    count: 1,
                }],
                vec![GidMap {
                    inside_gid: ROOT_PRV,
                    outside_gid: ROOT_PRV,
                    count: 1,
                }],
            )
            .status();

        if cmd_result.is_err() {
            println!("error is: {}", cmd_result.err().unwrap());
            process::exit(1);
        }
    } else {
        let cmd_result = Command::new("/bin/sh")
            .unshare(&[
                Namespace::Uts,
                Namespace::Ipc,
                Namespace::Pid,
                Namespace::Mount,
                Namespace::User,
                Namespace::Net,
            ])
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
            .status();

        if cmd_result.is_err() {
            println!("error is: {}", cmd_result.err().unwrap());
            process::exit(1);
        }
    }
}
