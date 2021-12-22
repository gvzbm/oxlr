Module(
    path: Path([Symbol("trivial")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 1,
                blocks: [
                    BasicBlock(
                        instrs: [
                            Return(LiteralInt(Integer(width: 64, signed: false, data: 0)))
                        ],
                        next_block: 0
                    )
                ]
            )
        )
    },
    imports: []
)
