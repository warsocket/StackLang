Stack Of Stacks
===============

Stack Of Stacks is an interpreted assembly like programming language that skirts the line of being a esoteric programming language by having a reaonably normal instruction set of 16 opcodes (all with 0 parameters) but a offbeat syntax using single character symbols.

It has been proven turing complete by simulating the Rule 110 cellular automata; doing so in only 372 instructions making it far from a turing tarpit.

Fun fact: since there are exactly 16 opcodes and fitting 2 opcodes in 1 byte (which is they bytecode you can generate with `--compile`) there are no invalid instructions when executing bytecode (which can be done using the `--bytecode` switch).
Futhermode, when not running in `--strict` mode, there are no exceptions so any random (non-)binary file is a well formed bytecode program which can be run and will keep runnning (unless it accidently explicitly executes HALT).

Model
-----
The languague consists of the following:
- Read only code memmory + code pointer(CP) pointing to current instruction
- 2 stacks where each item is a (signed)64-bits number
- 16 instruction 

Program execution works as follows `Monospace is pseudocode calrification`:
1. Read instruction from code memmory at code pointer. `instruction = code[CP]`
2. Execute the instruction and set code pointer to next instruction `interpreter_operations[instruction]()`
3. Set the code pointer to 1 instruction futher as it is now. `CP++`
4. Repeat from step 1 ad infinitum until system halts.

Instructions
------------
| Instruction | Mnemonic | Stacks Before | Stacks After |
|-|-|-|-|
| `!` | PUSH -1 | `[1,2]` `[3,4]` | `[1,2,-1]` `[3,4]` |
| `^` | XOR | `[1,2]` `[3,4]` | `[3]` `[3,4]` |
| `\|` | OR | `[1,2]` `[3,4]` | `[3]` `[3,4]` |
| `&` | AND | `[1,2]` `[3,4]` | `[0]` `[3,4]` |
| `+` | ADD | `[1,2]` `[3,4]` | `[3]` `[3,4]` |
| `-` | SUB | `[-1]` `[3,4]` | `[1,2]` `[3,4]` |
| `*` | MUL | `[2]` `[3,4]` | `[1,2]` `[3,4]` |
| `/` | DIV | `[0]` `[3,4]` | `[1,2]` `[3,4]` |
| `$` | SWAPSTACK | `[1,2]` `[3,4]` | `[3,4]` `[1,2]` |
| `~` | XCHANGE | `[1,2]` `[3,4]` | `[1,4]` `[3,2]` |
| `=` | DUP | `[1,2,2]` `[3,4]` | `[1,2]` `[3,4]` |
| `@` | JMPREL | `[1,2]` `[3,4]` | `[1]` `[3,4]` |
| `?` | READ | `[1,2]` `[3,4]` | `[1,2,stdin[0]]` `[3,4]` |
| `.` | WRITE | `[1,2]` `[3,4]` | `[1]` `[3,4]` |
| `0` | SHL0 | `[1,2]` `[3,4]` | `[1,4]` `[3,4]` |
| `1` | SHL1 | `[1,2]` `[3,4]` | `[1,5]` `[3,4]` |

