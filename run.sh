#!/bin/sh
cargo b --release
sudo setcap cap_net_admin=eip target/release/rust-tcp
target/release/rust-tcp &
pid=$!
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0
# trap 命令的作用​​:用于指定在脚本接收到特定信号时执行的命令 trap "command" SIGNAL...
trap "kill $pid" INT TERM
wait $pid
