Module(
    path: Path([Symbol("branching")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 32, signed: true)),
            FnBody(
                max_registers: 4,
                blocks: [
                    BasicBlock(
                        instrs: [
                            BinaryOp(Mul, Register(0), LiteralInt(3), LiteralInt(3)),
                            BinaryOp(Eq, Register(1), Reg(Register(0)), LiteralInt(9)),
                            Br(cond: Reg(Register(1)), if_true: 1, if_false: 2),
                        ],
                        next_block: 999
                    ),

                    BasicBlock(
                        instrs: [
                            BinaryOp(Mul, Register(2), LiteralInt(3), LiteralInt(3)),
                            BinaryOp(NEq, Register(3), Reg(Register(0)), LiteralInt(9)),
                            Br(cond: Reg(Register(3)), if_true: 2, if_false: 3)
                        ],
                        next_block: 999
                    ),

                    BasicBlock(
                        instrs: [
                            Return(LiteralInt(3333))
                        ],
                        next_block: 999
                    ),

                    BasicBlock(
                        instrs: [
                            Return(LiteralInt(0))
                        ],
                        next_block: 999
                    )
                ]
            )
        )
    },
    imports: []
)
