# Union File System

## What is Union file system

Union File System, or UnionFS for short, is a file system service designed for Linux, FreeBSD and NetBDS systems that unites other file systems into a single union mount point. It uses branches to "transparently" overwrite files and directories from different filesystems to form a single consistent filesystem. These branches are read-only or read-write, so when a write operation is performed to this virtualized federated filesystem, the system is actually writing to a new file. It looks like this virtualized federated file system can operate on any file, but it actually does not change the original file, because UnionFS uses an important resource management technique called copy-on-write.

copy-on-write, also called implicit sharing, is a resource management technique that enables efficient replication of modifiable resources. The idea is that if a resource is duplicated, but not modified in any way, it is not necessary to create a new resource immediately, and this resource can be shared between the old and new instances. The creation of a new resource occurs at the first operation, when the resource is modified. By sharing resources in this way, the consumption of unmodified resource replication can be significantly reduced, but it also adds a small amount of overhead when making resource modifications.

## AUFS

AUFS, known as Advanced Multi-Layered Unification Filesystem, is a complete rewrite of the earlier UnionFS version 1.x. It is intended for reliability and performance and introduces some new features, such as load balancing of writable branches. some implementations of AUFS have been incorporated into UnionFS version 2.x.

## How to use AUFS by Docker

AUFS was the first storage driver chosen by Docker. AUFS has the advantage of fast starting containers and efficient use of storage and memory. Until now, AUFS is still a storage driver type supported by Docker, but of course, not anymore, if it is the latest community version of Docker, it has replaced AUFS with overlay2, and now it is almost impossible to find one that uses AUFS, see https://docs.docker.com/storage/storagedriver/aufs-driver/ The following needs to be described how Docker uses AUFS to store images and containers.

### image layer and AUFS

Each Docker image is composed of a series of read-only layers. image layers are stored in the /var/lib/docker/aufs/diff directory of the Docker hosts filesystem. The var/lib/docker/aufs/layers directory stores the metadata of how the image layers are stacked.


If there are 4 layers in image A, there are 4 corresponding storage directories in the diff directory, so if we make an image B based on image A, the Dockerfile of image B is :

```dockerfile
FROM image:12.04
RUN echo "Hello world" > /tmp/newfile
```

Pass ``docker build -t <image_name> . ` command, then the final view through `docker history <image_name>` will show a total of 5 image layers, and the echo command we added is the top layer, which is equivalent to pressing each line of command into a layer stack. This new image takes up very little space, largely because it reuses the underlying base image layer, improving storage efficiency. Of course, if you look through the /var/lib/docker/aufs/diff directory, there are also 5 image layer directories at this time.

### container layer and AUFS


Docker uses AUFS's CoW technique to achieve image layer sharing and reduce disk space usage. coW means that AUFS needs to copy the entire file even if only a small part of the file is modified. This design can have some impact on container performance, especially if the files to be copied are large or located under many image layer stacks, or if AUFS needs to search deep into the directory structure tree. However, there is no need to worry about the transition. For a container, each image layer is copied at most once, and subsequent changes are made on the container layer where the first copy was made.

When you start a container, Docker creates a read-only init layer to store the contents of the container's environment, and a read-write layer to perform all write operations.

The container layer's mount directory is also /var/lib/docker/aufs/mnt. The container's metadata and configuration files are stored in the /var/lib/docker/containers/<container_id> directory. s read-write layer is stored in the /var/lib/docker/aufs/diff directory. This read-write layer exists even if the container is stopped, so restarting the container will not lose data, and only when a container is deleted will this read-write layer be deleted along with it.

```bash
mathxh@MathxH:~/Project/rocker$ sudo ls /var/lib/docker/overlay2
0270a5e2e44be1f8cd3f7d2fb69af383525baa609b05f51298bc1f9f87791193 6e9bb43d35de999754fa986fd717fedfe92aeba234da631354e7a48030b6d39a
0270a5e2e44be1f8cd3f7d2fb69af383525baa609b05f51298bc1f9f87791193-init 6ed463275f62b359f83cce87148cf62f8c6dea0ba756e796e8fa02b423997934
038adb3896e296d7d6d123e06404db9eeb9b5dfb282c5faf6f5fd226882e87a1 6ee846113d7c16018ec3c37c04a8cd8a824cd2f06851b2f3bd29734d3d152fe9
047d18937cb78316c1f3a59c28f18896af1f314e56e0469eb0077cf61a9fce41 72918772406b95619d3186b5fd4d542ec5e497b70add0e51d8f91a5d01f0328b
08d4b5f58a3afd1963364cb541d4dbd55bf5e6b1dc1cb04c26615ac7fbcd8f57 75b727d82fe7a8e3259f12e9a1dd9d325d3099ed7aba3ec7591a7cbe65c89d1e
0c656fedc1118362d4984625a5cd8b1257bcf584b529d2f2813e7a241fc80519 7761a0111c5ecb1fb38aec8734ab22f27915dc4c05699ead3568e24b234abc05
118c281520747e7269fc3011d835360beab0464bb94f817a29d3471b4c7f770c 7d55067bd90ac6eabe4260edd9ea873c044bfa0d6c1fb348702ba0671f83c335
11b39baec34d4ab67ef606c56fe4655edc9efd234c22fd4300096517df780d5f 7d55067bd90ac6eabe4260edd9ea873c044bfa0d6c1fb348702ba0671f83c335-init
1207205f606d1026de80367349dc61813eebd0a42f59602b6a693eaf0b98c23f 8686152f0c6fd08f829cec47bbc2c87cf76b2861f9bbf3cbbe22d95843e7b61f


```

The overlay2 storage driver is also similar to AUFS. if you need to see all the layers, then go to /var/lib/docker/aufs/mnt, a federated mount point, and it will overlay all the layer directories together to provide a unified attempt.

Finally, a few words about how AUFS removes a file for a container. To delete file1, AUFS will generate a .wh.file at the read-write layer of the container to hide all read-only layer file1 files.

## Create AUFS with mount command

```bash
mathxh@MathxH:~$ mkdir aufs
mathxh@MathxH:~$ cd aufs/
mathxh@MathxH:~/aufs$ mkdir container-layer
mathxh@MathxH:~/aufs$ mkdir image-layer1
mathxh@MathxH:~/aufs$ mkdir image-layer2
mathxh@MathxH:~/aufs$ mkdir image-layer3
mathxh@MathxH:~/aufs$ echo "I am container layer" > ./container-layer/container-layer.txt
mathxh@MathxH:~/aufs$ echo "I am image layer 1" > ./image-layer1/image-layer1.txt
mathxh@MathxH:~/aufs$ echo "I am image layer 2" > ./image-layer2/image-layer2.txt
mathxh@MathxH:~/aufs$ echo "I am image layer 3" > ./image-layer3/image-layer3.txt
mathxh@MathxH:~/aufs$ sudo mount -t aufs -o dirs=./container-layer:./image-layer1:./image-layer2:./image-layer3 none ./mnt
```

Finally you will see the container-layer.txt and image-layer.txt files appear in the mnt folder

## Create Overlay with mount command

`upperdir` option is container layer, `lowerdir` is image layer:

```bash
mount -t overlay overlay -o lowerdir=./image-layer1:./image-layer2:./image-layer3,upperdir=./container-layer  ./mnt
```

The working directory (workdir) needs to be an empty directory on the same filesystem mount as the upper directory.

- The lower directory can be read-only or could be an overlay itself.
- The upper directory is normally writable.
- The workdir is used to prepare files as they are switched between the layers.

The lower directory can actually be a list of directories separated by :, all changes in the merged directory are still reflected in upper. 

## Create OverlayFS with another overlayFS implementation

directly use https://github.com/containers/fuse-overlayfs 

```bash
sudo apt install fuse-overlayfs
mathxh@MathxH:~/overlayfs$ mkdir container-layer
mkdir work
mathxh@MathxH:~/overlayfs$ mkdir image-layer2
mathxh@MathxH:~/overlayfs$ mkdir image-layer3
mathxh@MathxH:~/overlayfs$ echo "I am container layer" > ./container-layer/container-layer.txt
mathxh@MathxH:~/overlayfs$ echo "I am image layer 1" > ./image-layer1/image-layer1.txt
-bash: ./image-layer1/image-layer1.txt: No such file or directory
mathxh@MathxH:~/overlayfs$ ls
container-layer  image-layer2  image-layer3
mathxh@MathxH:~/overlayfs$ mkdir image-layer1
mathxh@MathxH:~/overlayfs$ echo "I am image layer 1" > ./image-layer1/image-layer1.txt
mathxh@MathxH:~/overlayfs$ echo "I am image layer 2" > ./image-layer2/image-layer2.txt
mathxh@MathxH:~/overlayfs$ echo "I am image layer 3" > ./image-layer3/image-layer3.txt
sudo fuse-overlayfs -o  lowerdir=./image-layer1:./image-layer2:./image-layer3,upperdir=./container-layer,workdir=./work ./mnt -o allow_other=true
```

and you can run to test CoW feature for OverlayFS:

```bash
mathxh@MathxH:~/overlayfs$ cd mnt/
mathxh@MathxH:~/overlayfs/mnt$ ls
container-layer.txt  image-layer1.txt  image-layer2.txt  image-layer3.txt
mathxh@MathxH:~/overlayfs/mnt$ cd ..
mathxh@MathxH:~/overlayfs$ cat ./mnt/container-layer.txt
I am container layer
mathxh@MathxH:~/overlayfs$ echo "Fuck you  image 3" >> ./mnt/image-layer3.txt
mathxh@MathxH:~/overlayfs$ cat ./mnt/image-layer3.txt
I am image layer 3
Fuck you  image 3
mathxh@MathxH:~/overlayfs$ cat ./image-layer3/image-layer3.txt
I am image layer 3
mathxh@MathxH:~/overlayfs$ cat ./container-layer/
container-layer.txt  image-layer3.txt
mathxh@MathxH:~/overlayfs$ cat ./container-layer/image-layer3.txt
I am image layer 3
Fuck you  image 3
```

According to the overlayFS documentation the upperdir directory is writable and the lowerdir is read-only. So let's set lowerdir as our mirror layer and upperdir as our container layer. So you can see that when I manipulate the data of image-layer3.txt in the directory mnt, you will see that there is no change in that file in the mirror layer, the change reacts in the directory mnt and in the directory of the container layer, and there is an extra file called image-layer3.txt with additional text content in the directory of the container layer.

In other words, when trying to write to the mnt/image-layer3.txt file, overlayFS first looks for the file named image-layer3.txt in the mnt directory, copies it to the container layer's directory in upperdir, and then does the following to the image-layer3.txt in the container layer This is very similar to the AUFS system and is much more concise and clear.

## Create mergerfs with command

```bash
sudo apt install mergerfs
sudo mergerfs -o allow_other,use_ino,link_cow=true  ./image-layer1:./image-layer2:./image-layer3:./container-layer ./mnt
```

```txt
mathxh@MathxH:~/aufs/mnt$ tree .
.
├── container-layer.txt
├── image-layer1.txt
├── image-layer2.txt
└── image-layer3.txt

0 directories, 4 files
```

remove 

```bash
sudo umount ./mnt
```


This fs does not support CoW like overlayFS or AUFS.

## References:

- https://wiki.archlinux.org/title/Overlay_filesystem
- https://unix.stackexchange.com/questions/588627/how-do-i-merge-directories-read-only-using-overlayfs
- https://docs.docker.com/storage/storagedriver/overlayfs-driver/
