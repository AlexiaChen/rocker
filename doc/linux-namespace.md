# Linux Namespace

It is a feature provided by the Linux kernel that isolates a range of system resources such as PID (Process ID), User ID, Network, etc.

For example, if you use the namespace API to do UID-level isolation, that is, you can virtualize a namespace with a UID of n. In this namespace, the user has root privileges. But on the real physical machine, he is still the same user with UID n. This solves the isolation problem between users. By analogy, using namespace for PID level isolation, it is like each namespace is like a separate Linux computer, with its own init process (PID 1), and the PIDs of other processes are incremented in turn. For example, a parent namespace creates two child namespaces, and both child namespaces have init processes with PID 1. The processes of the child namespaces are mapped to the processes of the parent namespace, and the parent namespace can know the running state of each child namespace, while the child namespaces are isolated from each other.

Currently Linux provides 6 different types of Namespace:

- Mount Namespace   CLONE_NEWNS
- UTS Namespace  CLONE_NEWUTS
- IPC Namespace  CLONE_NEWIPC
- PID Namespace  CLONE_NEWPID
- Network Namespace CLONE_NEWNET
- User Namespace  CLONE_NEWUSER

The Namespace API uses three main system calls as follows:

- clone() to create a new process. The system call parameters determine which types of namespaces are created and their children are included in these namespaces
- unshare() removes a process from a namespace
- setns() adds the process to the namespace