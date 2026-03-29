# setsid

Run a program in a new session.

## Implemented

- Create a new session and exec the given command
- `-c` / `--ctty`: set the current terminal as the controlling terminal
- `-f` / `--fork`: always fork (even if not process group leader)
- `-w` / `--wait`: wait for the child and return its exit status
- Automatic fork detection when already a process group leader
