# RustySnout
A network monitor and control application written in Rust

## Installation

### Ubuntu-based Distros
```
sudo apt update
sudo apt install libsqlite3-dev
cargo install bandwhich
```

### Arch-based Distros
```
sudo pacman -S sqlite3
sudo pacman -S bandwhich
```


---

```
which bandwhich
```
Using the path returned by the previous command:
```
sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep /path/to/bandwhich
```
