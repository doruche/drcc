Unoptimized TAC:
[external]
fn main (void) -> int
code:
	gte	%[i32]t.2, $[i32]1, $[i32]3
	bz	%[i32]t.2, bra.2
	lte	%[i32]t.5, $[i32]2, $[i32]4
	bz	%[i32]t.5, bra.2
	mov	%[i32]t.1, $[i32]1
	jmp	bra.3
bra.2:
	mov	%[i32]t.1, $[i32]0
bra.3:
	bnz	%[i32]t.1, bra.0
	bnz	$[i32]5, bra.0
	mov	%[i32]t.0, $[i32]0
	jmp	bra.1
bra.0:
	mov	%[i32]t.0, $[i32]1
bra.1:
	ret %[i32]t.0
	ret $[i32]0

local vars:

[external]
fn basic (void) -> int
code:
	bnz	$[i32]2, bra.0
	sub	%[i32]t.2, $[i32]5, $[i32]4
	bnz	%[i32]t.2, bra.0
	mov	%[i32]t.0, $[i32]0
	jmp	bra.1
bra.0:
	mov	%[i32]t.0, $[i32]1
bra.1:
	mul	%[i32]t.8, $[i32]5, $[i32]9
	add	%[i32]t.6, $[i32]1, %[i32]t.8
	rem	%[i32]t.11, $[i32]2, $[i32]3
	sub	%[i32]t.5, %[i32]t.6, %[i32]t.11
	ret %[i32]t.5
	ret $[i32]0

local vars:


Optimized TAC:
[external]
fn basic (void) -> int
code:
	bnz	$[i32]2, bra.0
	sub	%[i32]t.2, $[i32]5, $[i32]4
	bnz	%[i32]t.2, bra.0
	mov	%[i32]t.0, $[i32]0
	jmp	bra.1
bra.0:
	mov	%[i32]t.0, $[i32]1
bra.1:
	mul	%[i32]t.8, $[i32]5, $[i32]9
	add	%[i32]t.6, $[i32]1, %[i32]t.8
	rem	%[i32]t.11, $[i32]2, $[i32]3
	sub	%[i32]t.5, %[i32]t.6, %[i32]t.11
	ret %[i32]t.5

local vars:

[external]
fn main (void) -> int
code:
	gte	%[i32]t.2, $[i32]1, $[i32]3
	bz	%[i32]t.2, bra.2
	lte	%[i32]t.5, $[i32]2, $[i32]4
	bz	%[i32]t.5, bra.2
	mov	%[i32]t.1, $[i32]1
	jmp	bra.3
bra.2:
	mov	%[i32]t.1, $[i32]0
bra.3:
	bnz	%[i32]t.1, bra.0
	bnz	$[i32]5, bra.0
	mov	%[i32]t.0, $[i32]0
	jmp	bra.1
bra.0:
	mov	%[i32]t.0, $[i32]1
bra.1:
	ret %[i32]t.0

local vars:


