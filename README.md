Stack Of Stacks
===============

Stack Of Stacks is an interpreted assembly like programming language that skirts the line of being a esoteric programming language by having a reaonably normal instruction set of 16 opcodes (all with 0 parameters) but a offbeat syntax using single character symbols.

It has been proven turing complete by simulating the Rule 110 cellular automata; doing so in only 372 instructions making it far from a turing tarpit.

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
