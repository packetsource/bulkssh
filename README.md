# bulkssh

An SSH utility for running parallel connections to multiple hosts at once to extract the output
of a command (or a series of commands). Written in Rust, and makes use of the Rust ecosystem
async framework-du-jour, Tokio as well as Russh (Rust SSH implementation, based originally on
Thrussh).

Usage:

```
$ bulkssh -c "uname -a" -c "lsb-release -a" unix1 unix2 unix3
```

Output lines are preceded by hostname and command, allowing easy grepping of the resultant
wall of text. Multiple commands start up a shell each time on the remote host, but the same
underlying SSH session is used. 

Limits can be place on the number of concurrent SSH sessions that will be made with `-n` in
order to preserve host resources, CPU, network and/or all of these.

SSH sessions are established directly. No ~/.ssh/config file is run, and no support is
currently included for proxying via a gateway host. 

Authentication is key-based with a default private key sought from `~/.ssh/id_ed25519`,
but alternate locations can be specified with `-I`, and alternate usernames can be
specified with `-u`. These are constant for all established sessions, however.