# Optimizations
I chose to work with compiling code into the ANF and then into an intermediate
representation similar to the one in class before doing optimizations. The metric measured was instruction count since I couldn't get a timer to work without having huge variance.

I implemented constant folding, constant propogation, dead code elimination, and some type checking analysis.

**IR:** The results of the program when compiled with the base IR form was much larger, especially since every temporary went into memory.

**avoid checking:** To avoid checking everything, specific values in the IR compiled to checking code dependent on known type of the IR value at compile time. For this consider the types of {num, bool, nil, var, input}. Minimal gains above reference implementation, many programs were still longer than reference since they used more stack for temps.

**folding:** With just folding, up to 10 or 20 inst were reduced. Test programs did not have many expanded constants so minimal improvement.

**dead code elimination:** With smarter type checks, dead code elimination and folding there was zero improvements. The smarter type checking already handled dead code by not inserting type checks for folded constants.

**constant propogation:** Running fold,eliminate,propogate in a loop until no more changes, then compiling using the checking with type known from the IR provided decent results.

# Results
Full stdout output in txt files
## great results
nested_arith3 went from 102 to 35 instructions.

Optimizations worked best with variable assignments and if statements that could be propogated.

## could do better
BST program went from 1115 to 1102 instructions

points program went from 465 to 443 instructions

input and heap dependent programs did not improve much.

# Use and attributions

Profiling infra from https://github.com/ucsd-compilers-s23/optimisations-starter

Base compiler implementation from https://github.com/ucsd-compilers-s23/forest-flame-starter

ANF, IR code is extended and modified from https://github.com/ucsd-compilers-s23/lecture1/tree/ir/src

Example of use:

```
→ cat tests/bigloop.snek
(let ((n input))
  (loop
    (if (= n 0) (break 100)
        (set! n (- n 1)))))
→ make profile

... lots of output ...
---- profile_bigloop stdout ----
valgrind is only available on Linux. Using usr/bin/time to gauge instructions instead (use it only for relative comparisons on your system)
Instructions executed: 3111162207

Instructions in generated .s: 66
Instructions by type in your generated assembly before linking is:
  30 mov
   5 push
   5 pop
   4 test
   4 call
   3 jne
   3 jmp
   2 sub
   2 je
   2 cmp
   1 xor
   1 ret
   1 or
   1 jo
   1 cmove
   1 add


perf is only available on Linux. Using usr/bin/time instead.
Time taken in ms (seconds on MacOS):
1 0.33
2 0.36
3 0.34
4 0.30
5 0.34
```

This prints

1. The number of instructions counted in the execution (counted by `callgrind`
on Linux or `/usr/bin/time -l` on OSX)
2. A summary of the instructions used (statically counted)
3. Several trials of wall-clock time for the program

Try to improve on what forest-flame can do!
