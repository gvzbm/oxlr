
Three main components: common IR, virtual machine and compiler.

# Common IR

Define a Rust crate that contains type definitions to represent IR. For now we can just pass these around by using `serde` and [msgpack](https://msgpack.org) serialization and writing it into a file. The actual program flow is represented using an SSA-based graph representation, which makes compilation and program analysis easier, and also aids any sort of future JIT compiling.

The fundamental unit of IR is a module, which contains any submodules, a collection of type and interface definitions, a collection of function definitions and interface implementations, a set of exported types and functions, and a set of imported modules. Modules are versioned with Semver.

# Virtual machine

The virtual machine takes an IR module, loads it, and executes the `start` function, if present.

To load a module, first load all submodules. Next, load all imported modules. Imported modules specify the version to load in typical Semver fashion. Imported modules will be searched for in the import search path. These should be cached in the VM and only loaded once.

# Compiler

The compiler will take some human-usable language and transform it into the common IR for use in the VM.
Ideally this language matches the IR and VM semantics fairly close.
