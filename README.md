# drcc
The name `drcc` means "doruche's C compiler". The compiler implements a small subset of C language. All the compiler, ranging from lexing and parsing in frontend, IR generation and optimization in middle-end, to RISC-V assembly emission in backend, is written in Rust from scratch, without any dependency on other libraries.

---
__注意__


出于一些原因这个项目近乎没办法继续开发了。一个主要问题是缺乏足够全面或者有效的测例，导致我很多时候几乎只能凭感觉编写代码，然后通过自制的一点相当贫瘠的测例来进行验证。总之目前这个编译器或许潜伏着不少神秘的bug也未可知。敬请各位多加小心啦~~~

## Features
- Tiny subset of C language
  - A miniature of type system: only `int` and `long` types
  - File-scope functions and variables: `extern` and `static` keywords are supported, allowing for modular programming and calling library functions
- Middle-end optimizations:
  - Constant folding
  - Dead-code elimination
  - Copy propagation
  - Dead-store elimination
- Backend code generation:
  - Emits RV64IM assembly code
- Clear Error Reporting:
  - Errors are split into 3 categories: Lexical, Syntactic, and Semantic errors.
  - Errors will be reported in bunches, not one at a time.
  - Accurate location information, including line number and column number. e.g.
  ```
  Parsing error: Ln 3, Col 4:11 Expected ';' after 'continue' statement.
  ```

## Compiler Structure
`drcc` takes a quite clear structure, which can be loosely divided into following stages:

1. **Lexing** (`lex`): The input C source code is tokenized into a stream of tokens.
2. **Parsing** (`ast`): The token stream is parsed into an abstract syntax tree (AST). After parsing, we got an `AstTopLevel` structure, which contains all the top-level declarations and definitions in the source code.
3. **Semantic Analysis** (`sem`): Name resolution, label resolution, and type checking/annotating are done here. Any semantic errors will be reported as well. After this stage, we got an `HirTopLevel` structure, which restructures the AST into a high-level intermediate representation (HIR), containing all semantic information we need later, and stripping away all unnecessary details (e.g. In-block function declarations).
4. **TAC Generation** (`tac`): HIR, a tree-style IR, is translated into a classical three-address code (TAC) representation, i.e. `TacTopLevel`, which is considered as `drcc`'s MIR (mid-level IR). From here on, we do not consider errors anymore, and the compiler is expected to be correct.<br/>
Some machine-independent optimizations can be applied to TAC code as well (`tac/opt`). These are all intra-procedural optimizations, which do not cross function boundaries.
5. **LIR Generation** (`lir`): We transform the TAC into a more machine-oriented low-level IR (LIR), i.e. `LirTopLevel`. This is the last IR before we emit the final assembly code. The LIR representations are quite closer to real RISC-V assembly code. For instance, the generalized instructions are transformed into concrete RISC-V instructions, like `TacBinary -> {Add, Addw, Sub, ...}`.<br/>
Basically, the LIR stage is composed of 4 parts:
    - Incomplete LIR generation (`lir/parse.rs`)
    - Register allocation (`lir/regalloc`) : We take a traditional graph coloring approach to allocate registers.
    - Spilling (`lir/spill.rs`) : The unallocated virtual registers are spilled to stack slots here.
    - Instruction Canonicalization (`lir/canonic`) : During previous stages, we didn't consider the restrictions of real RISC-V instructions, which is for the purity of each layers' logic. So we need to transform those invalid instrucions to valid ones here. e.g. <br/>
    ```
    add x1, x2, 42 -> addi  x1, x2, 42

    mv  -8(s0), 42 -> li    t5, 42      
                      sw    t5, -8(s0)
    // we use t5/t6 as scratch registers for temporary values
    ```
    The canoniclization are split into immediate-related and memory-replated fixings.
6. **Assembly Emission** (`asm`): `LirTopLevel` is translated into `AsmTopLevel` at this final stage, where all intermediate instructions (e.g. `IntermediateInsn::Prologue`) are transformed into a structured RISC-V assembly instructions. To emit these instructions is quite easy - just print them out.<br/>
Btw, here is an ideal place for peephole optimizations, which are not implemented yet. 
## Examples
See `testprogs` folder for some example C programs. 