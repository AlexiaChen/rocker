## Container Images

We have created a simple container with namespace and cgroups technology, but we will find that the directory inside the container is still the directory of the current running program, and if we run the mount command, we can see all the mount points inherited from the parent process, which seems to be different from the usual performance of the container, because it is missing such an important feature as mirroring. Docker image can be said to be very good design, it makes container delivery and migration easier, now we need to make a short answer to your image, and then let it run in the environment with the image.

### Use busybox as basic image for rocker

busybox includes lots of Unix tools and commands. we will use it as first file system in the running container.

get busybox's rootfs and use `docker export` make it into tar package.

```bash
mathxh@MathxH:~/Project/rocker$ docker pull busybox
Using default tag: latest
latest: Pulling from library/busybox
50e8d59317eb: Pull complete
Digest: sha256:d2b53584f580310186df7a2055ce3ff83cc0df6caacf1e3489bff8cf5d0af5d8
Status: Downloaded newer image for busybox:latest
docker.io/library/busybox:latest
mathxh@MathxH:~/Project/rocker$ docker run -d busybox top -b
11b35852c1fe7e09fdfa5201f3bc2b1bd55294b417af26f193f89695e6937f17
mathxh@MathxH:~/Project/rocker$ docker export -o busybox.tar 11b35852c1fe7e09fdfa5201f3bc2b1bd55294b417af26f193f89695e6937f17
mathxh@MathxH:~/Project/rocker$ docker exec -it 11b35852c1fe7e09fdfa5201f3bc2b1bd55294b417af26f193f89695e6937f17 /bin/sh
/ # ls
bin   dev   etc   home  proc  root  sys   tmp   usr   var
/ # cd home/
/home # ls
/home # cd ..
/ # ls -l
total 36
drwxr-xr-x    2 root     root         12288 Apr 13 00:24 bin
drwxr-xr-x    5 root     root           340 May 20 05:25 dev
drwxr-xr-x    1 root     root          4096 May 20 05:25 etc
drwxr-xr-x    2 nobody   nobody        4096 Apr 13 00:25 home
dr-xr-xr-x  228 root     root             0 May 20 05:25 proc
drwx------    1 root     root          4096 May 20 05:27 root
dr-xr-xr-x   11 root     root             0 May 20 05:25 sys
drwxrwxrwt    2 root     root          4096 Apr 13 00:25 tmp
drwxr-xr-x    3 root     root          4096 Apr 13 00:25 usr
drwxr-xr-x    4 root     root          4096 Apr 13 00:25 var
/ # exit
mathxh@MathxH:~/Project/rocker$ docker ps
CONTAINER ID   IMAGE     COMMAND    CREATED         STATUS         PORTS     NAMES
11b35852c1fe   busybox   "top -b"   3 minutes ago   Up 3 minutes             tender_goldberg
```