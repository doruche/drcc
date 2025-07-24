Unoptimized TAC:
[external]
fn main (int argc) -> int
code:
	mov	%[i32]foo.1, $[i32]3
	mov	%[i32]foo.1, $[i32]4
	call	yet_another, %[i32]t.2 with ($[i32]10)
	ret %[i32]t.2
	ret $[i32]0

local vars:
	int %foo.1;

[external]
fn call_huge (void) -> int
code:
	call	huge, %[i32]t.0 with ($[i32]1, $[i32]2, $[i32]3, $[i32]4, $[i32]5, $[i32]6, $[i32]7, $[i32]8, $[i64]9, $[i32]10, $[i32]0)
	mov	%[i32]val.0, %[i32]t.0
	ret %[i32]val.0
	ret $[i32]0

local vars:
	int %val.0;

[external]
fn blank (void) -> int
code:
	ret $[i32]0

local vars:

[external]
fn huge (int a, int b, int c, int d, int e, int f, int g, int h, long x, int y, int pad) -> int
code:
	add	%[i32]t.10, %[i32]a.0, %[i32]b.1
	add	%[i32]t.9, %[i32]t.10, %[i32]c.2
	add	%[i32]t.8, %[i32]t.9, %[i32]d.3
	add	%[i32]t.7, %[i32]t.8, %[i32]e.4
	add	%[i32]t.6, %[i32]t.7, %[i32]f.5
	add	%[i32]t.5, %[i32]t.6, %[i32]g.6
	add	%[i32]t.4, %[i32]t.5, %[i32]h.7
	sext	%[i64]t.3, %[i32]t.4
	add	%[i64]t.2, %[i64]t.3, %[i64]x.8
	sext	%[i64]t.11, %[i32]y.9
	add	%[i64]t.1, %[i64]t.2, %[i64]t.11
	trunc	%[i32]t.0, %[i64]t.1
	ret %[i32]t.0
	ret $[i32]0

local vars:

[external]
fn another (int a, int b) -> int
code:
	mov	%[i32]foo.2, $[i32]3
	add	%[i32]t.1, %[i32]a.0, %[i32]b.1
	ret %[i32]t.1
	ret $[i32]0

local vars:
	int %foo.2;

[external]
fn yet_another (int a) -> int
code:
	gt	%[i32]t.0, %[i32]a.0, $[i32]0
	bz	%[i32]t.0, bra.0
	sub	%[i32]t.4, %[i32]a.0, $[i32]1
	call	yet_another, %[i32]t.3 with (%[i32]t.4)
	add	%[i32]t.2, %[i32]a.0, %[i32]t.3
	ret %[i32]t.2
	jmp	bra.1
bra.0:
	ret $[i32]0
bra.1:
	ret $[i32]0

local vars:


Optimized TAC:
[external]
fn another (int a, int b) -> int
code:
	mov	%[i32]foo.2, $[i32]3
	add	%[i32]t.1, %[i32]a.0, %[i32]b.1
	ret %[i32]t.1

local vars:
	int %foo.2;

[external]
fn blank (void) -> int
code:
	ret $[i32]0

local vars:

[external]
fn huge (int a, int b, int c, int d, int e, int f, int g, int h, long x, int y, int pad) -> int
code:
	add	%[i32]t.10, %[i32]a.0, %[i32]b.1
	add	%[i32]t.9, %[i32]t.10, %[i32]c.2
	add	%[i32]t.8, %[i32]t.9, %[i32]d.3
	add	%[i32]t.7, %[i32]t.8, %[i32]e.4
	add	%[i32]t.6, %[i32]t.7, %[i32]f.5
	add	%[i32]t.5, %[i32]t.6, %[i32]g.6
	add	%[i32]t.4, %[i32]t.5, %[i32]h.7
	sext	%[i64]t.3, %[i32]t.4
	add	%[i64]t.2, %[i64]t.3, %[i64]x.8
	sext	%[i64]t.11, %[i32]y.9
	add	%[i64]t.1, %[i64]t.2, %[i64]t.11
	trunc	%[i32]t.0, %[i64]t.1
	ret %[i32]t.0

local vars:

[external]
fn call_huge (void) -> int
code:
	call	huge, %[i32]t.0 with ($[i32]1, $[i32]2, $[i32]3, $[i32]4, $[i32]5, $[i32]6, $[i32]7, $[i32]8, $[i64]9, $[i32]10, $[i32]0)
	mov	%[i32]val.0, %[i32]t.0
	ret %[i32]val.0

local vars:
	int %val.0;

[external]
fn yet_another (int a) -> int
code:
	gt	%[i32]t.0, %[i32]a.0, $[i32]0
	bz	%[i32]t.0, bra.0
	sub	%[i32]t.4, %[i32]a.0, $[i32]1
	call	yet_another, %[i32]t.3 with (%[i32]t.4)
	add	%[i32]t.2, %[i32]a.0, %[i32]t.3
	ret %[i32]t.2
bra.0:
	ret $[i32]0

local vars:

[external]
fn main (int argc) -> int
code:
	mov	%[i32]foo.1, $[i32]3
	mov	%[i32]foo.1, $[i32]4
	call	yet_another, %[i32]t.2 with ($[i32]10)
	ret %[i32]t.2

local vars:
	int %foo.1;


