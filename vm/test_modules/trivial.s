Module(
    path: Path([Symbol("trivial")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 32, signed: true)),
            FnBody(
                max_registers: 1,
                blocks: [
                    BasicBlock(
                        instrs: [
                            Return(LiteralInt(0))
                        ],
                        next_block: 0
                    )
                ]
            )
        )
    },
    imports: []
)
