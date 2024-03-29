Turing completeness proofs
==========================

By structured program theorem
-----------------------------
3 examples which satisfies the 3 requirements of structured program theory:

1. `sequence.sos` Executing one subprogram, and then another subprogram (sequence).
2. `branch.sos` Executing one of two subprograms according to the value of a boolean expression (selection).
3. `while.sos` Repeatedly executing a subprogram as long as a boolean expression is true (iteration).

See <https://en.wikipedia.org/wiki/Structured_program_theorem> for more information.

By simulating another known turing complete system: Rule 110
------------------------------------------------------------

If you can implement / simulate a turing complete system yout have proven that your system itself is turing complete.
This is because by induction you were then able to implement a universal turing machine in your own language.

`110.sos` implements the rule 110 cellular automaton, which has been proven to be turing complete

See <https://en.wikipedia.org/wiki/Rule_110> for more information.

