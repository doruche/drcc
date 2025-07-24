# usage: debug_run.sh <program> <args>
qemu-riscv64 -g 1234 $1 -- $2