# logchuck

A simple CLI application designed to scan a directory of logfiles and allow un-zipping and merging logfiles into an `all.log` easily.

## Features

### Unzip rotating logs

`logchuck` will detect and unzip logfiles, no need to unzip older logs:

```
NetworkSocket_u3000.log
NetworkSocket_u3000.20230201.log.tar.gz
NetworkSocket_u3000.20230101.log.tar.gz
NetworkSocket_u3000.20221201.log.tar.gz
```

### Select which logfiles to see

To see HTTP traffic, you can select only logfiles which apply:

```
[ ] NetworkSocket_u3000.log
[x] NetworkSocket_t80.log
[x] NetworkSocket_u80.log
```
