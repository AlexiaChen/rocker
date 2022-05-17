///////////////////////////////////////////////////////// UTS
// mathxh@MathxH:~/Project/rocker/target/debug$ sudo ./ns
// $ pstree -pl
// init(1)─┬─init(12)───init(13)─┬─bash(14)───sudo(6780)───ns(6781)───sh(6782)───pstree(6831)
//         │                     └─sshd(73)
//         ├─init(88)───init(89)───sh(90)───sh(91)───sh(97)───node(99)─┬─node(110)─┬─{node}(111)
//         │                                                           │           ├─{node}(112)
//         │                                                           │           ├─{node}(113)
//         │                                                           │           ├─{node}(114)
//         │                                                           │           ├─{node}(115)
//         │                                                           │           ├─{node}(117)
//         │                                                           │           ├─{node}(133)
//         │                                                           │           ├─{node}(134)
//         │                                                           │           ├─{node}(135)
//         │                                                           │           └─{node}(136)
//         │                                                           ├─node(154)─┬─gopls(297)─┬─{gopls}(308)
//         │                                                           │           │            ├─{gopls}(309)
//         │                                                           │           │            ├─{gopls}(310)
//         │                                                           │           │            ├─{gopls}(311)
//         │                                                           │           │            ├─{gopls}(312)
//         │                                                           │           │            ├─{gopls}(313)
//         │                                                           │           │            ├─{gopls}(314)
//         │                                                           │           │            ├─{gopls}(321)
//         │                                                           │           │            ├─{gopls}(322)
//         │                                                           │           │            ├─{gopls}(324)
//         │                                                           │           │            ├─{gopls}(359)
//         │                                                           │           │            └─{gopls}(589)
//         │                                                           │           ├─{node}(155)
//         │                                                           │           ├─{node}(156)
//         │                                                           │           ├─{node}(157)
//         │                                                           │           ├─{node}(158)
//         │                                                           │           ├─{node}(159)
//         │                                                           │           ├─{node}(160)
//         │                                                           │           ├─{node}(161)
//         │                                                           │           ├─{node}(162)
//         │                                                           │           ├─{node}(163)
//         │                                                           │           ├─{node}(164)
//         │                                                           │           └─{node}(183)
//         │                                                           ├─node(165)─┬─{node}(166)
//         │                                                           │           ├─{node}(167)
//         │                                                           │           ├─{node}(168)
//         │                                                           │           ├─{node}(169)
//         │                                                           │           ├─{node}(170)
//         │                                                           │           ├─{node}(171)
//         │                                                           │           ├─{node}(172)
//         │                                                           │           ├─{node}(173)
//         │                                                           │           ├─{node}(174)
//         │                                                           │           └─{node}(175)
//         │                                                           ├─node(1141)─┬─rust-analyzer-x(3986)─┬─rust-analyzer-x(4022)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3987)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3988)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3989)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3990)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3991)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3992)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3993)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3994)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3995)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3996)
//         │                                                           │            │                       ├─{rust-analyzer-x}(3997)
//         │                                                           │            │                       └─{rust-analyzer-x}(5266)
//         │                                                           │            ├─rust-analyzer-x(5213)─┬─{rust-analyzer-x}(5214)
//         │                                                           │            │                       └─{rust-analyzer-x}(5215)
//         │                                                           │            ├─{node}(1142)
//         │                                                           │            ├─{node}(1143)
//         │                                                           │            ├─{node}(1144)
//         │                                                           │            ├─{node}(1145)
//         │                                                           │            ├─{node}(1146)
//         │                                                           │            ├─{node}(1147)
//         │                                                           │            ├─{node}(1155)
//         │                                                           │            ├─{node}(1156)
//         │                                                           │            ├─{node}(1157)
//         │                                                           │            ├─{node}(1158)
//         │                                                           │            └─{node}(1163)
//         │                                                           ├─node(1148)─┬─{node}(1149)
//         │                                                           │            ├─{node}(1150)
//         │                                                           │            ├─{node}(1151)
//         │                                                           │            ├─{node}(1152)
//         │                                                           │            ├─{node}(1153)
//         │                                                           │            ├─{node}(1154)
//         │                                                           │            ├─{node}(1159)
//         │                                                           │            ├─{node}(1160)
//         │                                                           │            ├─{node}(1161)
//         │                                                           │            └─{node}(1162)
//         │                                                           ├─{node}(100)
//         │                                                           ├─{node}(101)
//         │                                                           ├─{node}(102)
//         │                                                           ├─{node}(103)
//         │                                                           ├─{node}(104)
//         │                                                           ├─{node}(105)
//         │                                                           ├─{node}(106)
//         │                                                           ├─{node}(107)
//         │                                                           ├─{node}(108)
//         │                                                           └─{node}(109)
//         ├─init(727)───init(728)───bash(729)───v2ray(1091)─┬─{v2ray}(1092)
//         │                                                 ├─{v2ray}(1093)
//         │                                                 ├─{v2ray}(1094)
//         │                                                 ├─{v2ray}(1095)
//         │                                                 ├─{v2ray}(1096)
//         │                                                 ├─{v2ray}(1097)
//         │                                                 ├─{v2ray}(1098)
//         │                                                 ├─{v2ray}(1099)
//         │                                                 ├─{v2ray}(1100)
//         │                                                 ├─{v2ray}(1101)
//         │                                                 ├─{v2ray}(1136)
//         │                                                 ├─{v2ray}(1185)
//         │                                                 └─{v2ray}(1661)
//         └─{init}(8)
// $ echo $$
// 6782
// $ readlink /proc/6781/ns/uts
// $ readlink /proc/6782/ns/uts
// uts:[4026532204]

//////////////////////////////////////////////  IPC
// mathxh@MathxH:~$ ipcs -q

// ------ Message Queues --------
// key        msqid      owner      perms      used-bytes   messages

// mathxh@MathxH:~$ ipcmk -Q
// Message queue id: 0

// mathxh@MathxH:~/Project/rocker/target/debug$ ipcs -q

// ------ Message Queues --------
// key        msqid      owner      perms      used-bytes   messages
// 0xc8cf24d8 0          mathxh     644        0            0

// mathxh@MathxH:~/Project/rocker/target/debug$ sudo ./ns
// $ ipcs -q

// ------ Message Queues --------
// key        msqid      owner      perms      used-bytes   messages

/////////////////////////////////////////////////////////////////// PID
// mathxh@MathxH:~$ pstree -pl
// init(1)─┬─init(12)───init(13)─┬─bash(14)───sudo(1465)───ns(1466)───sh(1467)
//         │                     └─sshd(73)
//         ├─init(88)───init(89)───sh(90)───sh(91)───sh(97)───node(99)─┬─node(114)─┬─{node}(115)
//         │                                                           │           ├─{node}(116)
//         │                                                           │           ├─{node}(117)
//         │                                                           │           ├─{node}(118)
//         │                                                           │           ├─{node}(119)
//         │                                                           │           ├─{node}(121)
//         │                                                           │           ├─{node}(136)
//         │                                                           │           ├─{node}(138)
//         │                                                           │           ├─{node}(139)
//         │                                                           │           └─{node}(140)
//         │                                                           ├─node(194)─┬─node(715)─┬─{node}(716)
//         │                                                           │           │           ├─{node}(717)
//         │                                                           │           │           ├─{node}(718)
//         │                                                           │           │           ├─{node}(719)
//         │                                                           │           │           ├─{node}(720)
//         │                                                           │           │           └─{node}(721)
//         │                                                           │           ├─rust-analyzer-x(263)─┬─rust-analyzer-x(304)
//         │                                                           │           │                      ├─{rust-analyzer-x}(268)
//         │                                                           │           │                      ├─{rust-analyzer-x}(269)
//         │                                                           │           │                      ├─{rust-analyzer-x}(274)
//         │                                                           │           │                      ├─{rust-analyzer-x}(275)
//         │                                                           │           │                      ├─{rust-analyzer-x}(276)
//         │                                                           │           │                      ├─{rust-analyzer-x}(277)
//         │                                                           │           │                      ├─{rust-analyzer-x}(278)
//         │                                                           │           │                      ├─{rust-analyzer-x}(279)
//         │                                                           │           │                      ├─{rust-analyzer-x}(280)
//         │                                                           │           │                      ├─{rust-analyzer-x}(281)
//         │                                                           │           │                      ├─{rust-analyzer-x}(282)
//         │                                                           │           │                      ├─{rust-analyzer-x}(334)
//         │                                                           │           │                      ├─{rust-analyzer-x}(659)
//         │                                                           │           │                      ├─{rust-analyzer-x}(660)
//         │                                                           │           │                      ├─{rust-analyzer-x}(661)
//         │                                                           │           │                      ├─{rust-analyzer-x}(662)
//         │                                                           │           │                      ├─{rust-analyzer-x}(663)
//         │                                                           │           │                      ├─{rust-analyzer-x}(664)
//         │                                                           │           │                      ├─{rust-analyzer-x}(665)
//         │                                                           │           │                      └─{rust-analyzer-x}(666)
//         │                                                           │           ├─{node}(195)
//         │                                                           │           ├─{node}(196)
//         │                                                           │           ├─{node}(197)
//         │                                                           │           ├─{node}(198)
//         │                                                           │           ├─{node}(199)
//         │                                                           │           ├─{node}(200)
//         │                                                           │           ├─{node}(201)
//         │                                                           │           ├─{node}(202)
//         │                                                           │           ├─{node}(203)
//         │                                                           │           ├─{node}(204)
//         │                                                           │           └─{node}(216)
//         │                                                           ├─node(205)─┬─{node}(206)
//         │                                                           │           ├─{node}(207)
//         │                                                           │           ├─{node}(208)
//         │                                                           │           ├─{node}(209)
//         │                                                           │           ├─{node}(210)
//         │                                                           │           ├─{node}(211)
//         │                                                           │           ├─{node}(212)
//         │                                                           │           ├─{node}(213)
//         │                                                           │           ├─{node}(214)
//         │                                                           │           └─{node}(215)
//         │                                                           ├─node(941)─┬─gopls(1042)─┬─{gopls}(1044)
//         │                                                           │           │             ├─{gopls}(1045)
//         │                                                           │           │             ├─{gopls}(1046)
//         │                                                           │           │             ├─{gopls}(1047)
//         │                                                           │           │             ├─{gopls}(1048)
//         │                                                           │           │             ├─{gopls}(1057)
//         │                                                           │           │             ├─{gopls}(1067)
//         │                                                           │           │             ├─{gopls}(1068)
//         │                                                           │           │             ├─{gopls}(1069)
//         │                                                           │           │             ├─{gopls}(1070)
//         │                                                           │           │             └─{gopls}(1071)
//         │                                                           │           ├─{node}(942)
//         │                                                           │           ├─{node}(943)
//         │                                                           │           ├─{node}(944)
//         │                                                           │           ├─{node}(945)
//         │                                                           │           ├─{node}(946)
//         │                                                           │           ├─{node}(947)
//         │                                                           │           ├─{node}(948)
//         │                                                           │           ├─{node}(949)
//         │                                                           │           ├─{node}(950)
//         │                                                           │           ├─{node}(951)
//         │                                                           │           └─{node}(963)
//         │                                                           ├─node(952)─┬─{node}(953)
//         │                                                           │           ├─{node}(954)
//         │                                                           │           ├─{node}(955)
//         │                                                           │           ├─{node}(956)
//         │                                                           │           ├─{node}(957)
//         │                                                           │           ├─{node}(958)
//         │                                                           │           ├─{node}(959)
//         │                                                           │           ├─{node}(960)
//         │                                                           │           ├─{node}(961)
//         │                                                           │           └─{node}(962)
//         │                                                           ├─{node}(104)
//         │                                                           ├─{node}(105)
//         │                                                           ├─{node}(106)
//         │                                                           ├─{node}(107)
//         │                                                           ├─{node}(108)
//         │                                                           ├─{node}(109)
//         │                                                           ├─{node}(110)
//         │                                                           ├─{node}(111)
//         │                                                           ├─{node}(112)
//         │                                                           └─{node}(113)
//         ├─init(821)───init(822)───bash(823)───pstree(1468)
//         └─{init}(8)

// mathxh@MathxH:~/Project/rocker/target/debug$ sudo ./ns
// [sudo] password for mathxh:
// $ echo $$                       // means show current PID
// 1

// Mount
//
// mathxh@MathxH:~/Project/rocker/target/debug$ sudo ./ns
// # echo $$
// 1
// # ls /proc
// 1    14    1588  201  73  91   993        bus        consoles  diskstats    filesystems  ioports   key-users    kpagecount  mdstat   mounts        partitions   softirqs  sysvipc      uptime       zoneinfo
// 117  1584  161   332  88  97   994        cgroups    cpuinfo   dma          fs           irq       keys         kpageflags  meminfo  mtrr          sched_debug  stat      thread-self  version
// 12   1585  172   361  89  99   acpi       cmdline    crypto    driver       interrupts   kallsyms  kmsg         loadavg     misc     net           schedstat    swaps     timer_list   vmallocinfo
// 13   1586  189   425  90  992  buddyinfo  config.gz  devices   execdomains  iomem        kcore     kpagecgroup  locks       modules  pagetypeinfo  self         sys       tty          vmstat
// # mount -t proc proc /proc
// # ls /proc
// 1     buddyinfo  cmdline    cpuinfo  diskstats  execdomains  interrupts  irq       key-users  kpagecgroup  loadavg  meminfo  mounts  pagetypeinfo  schedstat  stat   sysvipc      tty      vmallocinfo
// 4     bus        config.gz  crypto   dma        filesystems  iomem       kallsyms  keys       kpagecount   locks    misc     mtrr    partitions    self       swaps  thread-self  uptime   vmstat
// acpi  cgroups    consoles   devices  driver     fs           ioports     kcore     kmsg       kpageflags   mdstat   modules  net     sched_debug   softirqs   sys    timer_list   version  zoneinfo
// # ps -ef
// UID        PID  PPID  C STIME TTY          TIME CMD
// root         1     0  0 20:12 pts/1    00:00:00 /bin/sh
// root         5     1  0 20:13 pts/1    00:00:00 ps -ef

//////////////////////////////  User
// mathxh@MathxH:~/Project/rocker/target/debug$ su root
// Password:
// root@MathxH:/home/mathxh/Project/rocker/target/debug# id
// uid=0(root) gid=0(root) groups=0(root)
// root@MathxH:/home/mathxh/Project/rocker/target/debug# ./ns
// $ id
// uid=65534(nobody) gid=65534(nogroup) groups=65534(nogroup)
// $

//////////////////////////////// Network
// mathxh@MathxH:~/Project/rocker/target/debug$ ifconfig
// eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
//         inet 172.21.111.53  netmask 255.255.240.0  broadcast 172.21.111.255
//         inet6 fe80::215:5dff:fe7f:b325  prefixlen 64  scopeid 0x20<link>
//         ether 00:15:5d:7f:b3:25  txqueuelen 1000  (Ethernet)
//         RX packets 100354  bytes 63888210 (63.8 MB)
//         RX errors 0  dropped 0  overruns 0  frame 0
//         TX packets 109753  bytes 289025475 (289.0 MB)
//         TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

// lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536
//         inet 127.0.0.1  netmask 255.0.0.0
//         inet6 ::1  prefixlen 128  scopeid 0x10<host>
//         loop  txqueuelen 1000  (Local Loopback)
//         RX packets 44  bytes 2200 (2.2 KB)
//         RX errors 0  dropped 0  overruns 0  frame 0
//         TX packets 44  bytes 2200 (2.2 KB)
//         TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

// mathxh@MathxH:~/Project/rocker/target/debug$ sudo ./ns
// [sudo] password for mathxh:
// usage: ./ns 0(root in container) or ./ns 1(no root in container)
// mathxh@MathxH:~/Project/rocker/target/debug$ sudo ./ns 1
// $ ifconfig
// $

use std::{env, process};
use unshare::{Command, GidMap, Namespace, UidMap};

const ROOT_PRV: u32 = 0;
const NO_ROOT_PRV: u32 = 1;

fn main() {
    if env::args().len() != 2 {
        println!("usage: ./ns 0(root user in container) or ./ns 1(no root user in container)");
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
