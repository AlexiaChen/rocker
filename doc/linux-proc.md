# Introduction to Linux proc File System

The /proc file system under Linux is provided by the kernel, it is not really a file system, it only contains information about the system runtime (system memory, mount device information, some hardware configuration, etc.), it exists only in memory and does not occupy hard disk space. It takes the form of a file system and provides an interface for accessing kernel data operations. In fact, many system tools simply read the contents of a file system, such as lsmod, which actually reads /proc/modules.

Several important parts will be described below.

```txt
/proc/N                       Information about processes with PID N
/proc/N/cmdline               Process start command
/proc/N/cwd                   Link to the current working directory of the process
/proc/N/environ               List of process environment variables
/proc/N/exe                   Link to the process's execution command file
/proc/N/fd                    contains all the process-related file descriptors
/proc/N/maps                  Memory mapping information related to the process
/proc/N/mem                   refers to the memory held by the process and is not readable
/proc/N/root                  Links to the root of the process
/proc/N/stat                  Status of the process
/proc/N/statm                 Status of memory used by the process
/proc/N/status                process status information, more readable than /proc/N/stat
/proc/self/                   link to the currently running process
```