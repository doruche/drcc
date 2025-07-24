Unoptimized TAC:
[external]
fn main (void) -> int
code:
	mov	%[i32]i.0, $[i32]0
con.0:
	ls	%[i32]t.1, %[i32]i.0, $[i32]5
	bz	%[i32]t.1, brk.0
	mov	%[i32]j.1, $[i32]0
bra.0:
	ls	%[i32]t.4, %[i32]j.1, $[i32]5
	bz	%[i32]t.4, brk.1
	eq	%[i32]t.6, %[i32]j.1, $[i32]2
	bz	%[i32]t.6, bra.1
	jmp	con.1
	jmp	bra.2
bra.1:
	eq	%[i32]t.8, %[i32]j.1, $[i32]4
	bz	%[i32]t.8, bra.3
	jmp	brk.1
bra.3:
bra.2:
con.1:
	add	%[i32]t.10, %[i32]j.1, $[i32]1
	mov	%[i32]j.1, %[i32]t.10
	jmp	bra.0
brk.1:
	add	%[i32]t.12, %[i32]i.0, $[i32]1
	mov	%[i32]i.0, %[i32]t.12
	eq	%[i32]t.14, %[i32]i.0, $[i32]3
	bz	%[i32]t.14, bra.4
	jmp	brk.0
bra.4:
	jmp	con.0
brk.0:
	ret %[i32]i.0
	ret $[i32]0

local vars:
	int %i.0;
	int %j.1;


Optimized TAC:
[external]
fn main (void) -> int
code:
	mov	%[i32]i.0, $[i32]0
con.0:
	ls	%[i32]t.1, %[i32]i.0, $[i32]5
	bz	%[i32]t.1, brk.0
	mov	%[i32]j.1, $[i32]0
bra.0:
	ls	%[i32]t.4, %[i32]j.1, $[i32]5
	bz	%[i32]t.4, brk.1
	eq	%[i32]t.6, %[i32]j.1, $[i32]2
	bz	%[i32]t.6, bra.1
	jmp	con.1
bra.1:
	eq	%[i32]t.8, %[i32]j.1, $[i32]4
	bz	%[i32]t.8, bra.3
	jmp	brk.1
bra.3:
con.1:
	add	%[i32]t.10, %[i32]j.1, $[i32]1
	mov	%[i32]j.1, %[i32]t.10
	jmp	bra.0
brk.1:
	add	%[i32]t.12, %[i32]i.0, $[i32]1
	mov	%[i32]i.0, %[i32]t.12
	eq	%[i32]t.14, %[i32]i.0, $[i32]3
	bz	%[i32]t.14, bra.4
	jmp	brk.0
bra.4:
	jmp	con.0
brk.0:
	ret %[i32]i.0

local vars:
	int %i.0;
	int %j.1;


