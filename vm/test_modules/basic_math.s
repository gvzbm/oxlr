Module(
    path: Path([Symbol("basic_math")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 2,
                blocks: [
                    BasicBlock(
                        instrs: [
                            BinaryOp(Mul, Register(0), LiteralInt(Integer(width: 64, signed: false, data: 3)), LiteralInt(Integer(width: 64, signed: false, data: 3))),
                            BinaryOp(Sub, Register(1), Reg(Register(0)), LiteralInt(Integer(width: 64, signed: false, data: 9))),
                            Return(Reg(Register(1)))
                        ],
                        next_block: 0
                    )
                ]
            )
        )
    },
    imports: []
)
