use unshare::{Command, Namespace, UidMap, GidMap};


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

fn main() {
    
    let cmd_result = Command::new("/bin/sh")
    .unshare(&[Namespace::Uts, Namespace::Ipc, Namespace::Pid])
    .set_id_maps(vec![UidMap{inside_uid:1, outside_uid:0, count: 1}], vec![GidMap{inside_gid:1, outside_gid:0, count: 1}])
    .status();

    if cmd_result.is_err() {
        println!("error is: {}", cmd_result.err().unwrap() )
    }

}
