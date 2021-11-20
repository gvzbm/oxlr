Module(
    path: Path([Symbol("phi_nodes")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 32, signed: true)),
            FnBody(
                max_registers: 5,
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
                            LoadImm(Register(2), LiteralInt(0))
                        ],
                        next_block: 3
                    ),

                    BasicBlock(
                        instrs: [
                            LoadImm(Register(3), LiteralInt(3333))
                        ],
                        next_block: 3
                    ),

                    BasicBlock(
                        instrs: [
                            Phi(Register(4), {
                                1: Reg(Register(2)),
                                2: Reg(Register(3))
                            }),
                            Return(Reg(Register(4)))
                        ],
                        next_block: 3
                    )
                ]
            )
        )
    },
    imports: []
)
