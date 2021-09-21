# Linux Namespace

It is a feature provided by the Linux kernel that isolates a range of system resources such as PID (Process ID), User ID, Network, etc.

For example, if you use the namespace API to do UID-level isolation, that is, you can virtualize a namespace with a UID of n. In this namespace, the user has root privileges. But on the real physical machine, it is still the same user with UID n. This solves the isolation problem between users. By analogy, using namespace for PID level isolation, it is like each namespace is like a separate Linux computer, with its own init process (PID 1), and the PIDs of other processes are incremented in turn. For example, a parent namespace creates two child namespaces, and both child namespaces have init processes with PID 1. The processes of the child namespaces are mapped to the processes of the parent namespace, and the parent namespace can know the running state of each child namespace, while the child namespaces are isolated from each other.

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

## UTS

The UTS namespace is mainly used to isolate the nodename and domainname system identifiers. In the UTS namespace, each namespace is allowed to have its own hostname.

## IPC

The IPC namespace is used to isolate the System V IPC and POSIX message queues. Each IPC namespace has its own System V IPC and POXIS message queues.

## PID

The PID namespace is used to isolate process IDs. It is understandable that inside the docker container, using ps -ef, you will often find that inside the container, the process running in the foreground has PID 1, but outside the container, using ps -ef, you will find that the same process has a different PID, and this is the PID namespace to do This is what the PID namespace does.

## Mount

The Mount namespace is used to isolate the view of mount points seen by individual processes. The file system hierarchy seen by processes in different namespaces is different. Calls to mount() and unmont() in the Mount namespace only affect the filesystem in the current namespace, and have no effect on the global filesystem. Similar to chroot(), but more flexible and secure. This namespace was the first namespace type implemented in Linux, so the parameter is CLONE_NEWNS. docker's Volume also takes advantage of this feature.

## User

The User namespace mainly isolates the user's user group ID. i.e., the UID and GID of a process can be different inside and outside the namespace. It is common to create a User namespace on the host as a non-root user, and then map it to the root user inside the User namespace. This means that the process has root privileges inside the User namespace, but not outside the User namespace

## NetWork

Network namespace is a namespace used to isolate network devices, IP address ports, and other network stacks. It allows each container to have its own independent (virtual) network device, and applications within the container can be bound to their own ports, and the ports within each namespace will not conflict with each other. After building a bridge on the host, it is easy to implement inter-container communication, and applications on different containers can use the same ports.