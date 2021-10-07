
Things to have in general:
- IR
    - strongly typed
    - SSA
    - but more at JVM level than LLVM level
- managed memory?
- types: sum, product, integers, floats, chars, strings, bool, reference?
- do we want composite value types?
- need to keep track of APIs
- object style associated functions vs trait style associated functions?
- operator overloads?

Things that would be cool:

- API projection into different styles for different syntaxes to access same API

Components:

- Some kind of compiler for a useable nice language (possibly an imperative one)
    - Compile into IR
    - A functional style compiler as well?
    - Could just do a lisp at first

- Virtual machine
    - just interpret IR for now
    - possibly also JIT compile? (could use LLVM or Cranelift)
    - garbage collection?

- Common library to represent, (de)ser IR

at some point:

- Partial evaluator
    - really for the full power this needs to be self-hosting
    - operate on IR

- Language Server for IDEs?
    - requires partial parsing

- Program analysis?

Self-host status:
    - Ideally anything partial eval wise is self-hosting
    - It would be really cool to have self-hosted compiler, but might be a long run target
    - Can start by crafting a second language using the first
    - VM is unlikely to be self-hosted, I suppose it could be attempted but it's a long target
