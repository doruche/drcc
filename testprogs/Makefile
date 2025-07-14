CROSS = riscv64-unknown-elf-
CC = $(CROSS)gcc
CFLAGS = -Wall

SRCS = $(wildcard *.c)
ASMS = $(SRCS:.c=.S)

.PHONY: all clean

all: $(ASMS)

%.S: %.c
	$(CC) $(CFLAGS) -S -o $@ $<

clean:
	@rm -f $(ASMS)