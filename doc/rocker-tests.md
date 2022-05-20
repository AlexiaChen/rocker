## Rocker Usage for tests

```bash
mathxh@MathxH:~/Project/rocker/target/debug$ sudo RUST_LOG=trace ./rocker  run --tty -m 100m "stress --vm-bytes 200m --vm-keep -m 1"
 INFO  rocker > hello rocker
 DEBUG rocker > Match Cmd: run
 DEBUG rocker > rocker run  tty:true, cmd:stress --vm-bytes 200m --vm-keep -m 1
 INFO  rocker > hello rocker
 DEBUG rocker > Match Cmd: init
 DEBUG rocker > rocker init cmd:stress --vm-bytes 200m --vm-keep -m 1
 TRACE rocker > waiting parent finish
stress: info: [1] dispatching hogs: 0 cpu, 0 io, 1 vm, 0 hdd
stress: FAIL: [1] (415) <-- worker 2 got signal 9
stress: WARN: [1] (417) now reaping child worker processes
stress: FAIL: [1] (421) kill error: No such process
stress: FAIL: [1] (451) failed run completed in 0s
 TRACE rocker > parent process wait finished exit status is exited with code 1
mathxh@MathxH:~/Project/rocker/target/debug$ sudo RUST_LOG=trace ./rocker  run --tty -m 100m "stress --vm-bytes 50m --vm-keep -
m 1"
 INFO  rocker > hello rocker
 DEBUG rocker > Match Cmd: run
 DEBUG rocker > rocker run  tty:true, cmd:stress --vm-bytes 50m --vm-keep -m 1
 INFO  rocker > hello rocker
 DEBUG rocker > Match Cmd: init
 DEBUG rocker > rocker init cmd:stress --vm-bytes 50m --vm-keep -m 1
 TRACE rocker > waiting parent finish
stress: info: [1] dispatching hogs: 0 cpu, 0 io, 1 vm, 0 hdd
```

As you can see above, if the stress command line exceeds the memory limit defined by rocker's cgroup, it is killed by signal 9. If it is less than the memory limit defined by the cgroup, then it will keep running.

