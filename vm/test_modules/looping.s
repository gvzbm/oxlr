Module(
    path: Path([Symbol("looping")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 5,
                blocks: [
                    BasicBlock(
                        instrs: [
                            LoadImm(Register(0), LiteralInt(Integer(width: 64, signed: false, data: 2)))
                        ], // simply fall through to the loop body
                        next_block: 1
                    ),
                    BasicBlock(
                        instrs: [
                            Phi(Register(1), {
                                0: Reg(Register(0)),
                                1: Reg(Register(2))
                            }),
                            BinaryOp(Mul, Register(2), Reg(Register(1)),
                                LiteralInt(Integer(width: 64, signed: false, data: 3))),
                            BinaryOp(Eq, Register(3), Reg(Register(2)),
                                LiteralInt(Integer(width: 64, signed: false, data: 18))),
                            Br(cond: Reg(Register(3)),
                                if_true: 2,
                                if_false: 1)
                        ],
                        next_block: 0
                    ),
                    BasicBlock(
                        instrs: [
                            // do we need a phi node here for %2? seems unnecessary since we know
                            // it's not possible to get to this block without first setting %2
                            BinaryOp(Sub, Register(4), Reg(Register(2)),
                                LiteralInt(Integer(width: 64, signed: false, data: 18))),
                            Return(Reg(Register(4)))
                        ],
                        next_block: 99
                    )
                ]
            )
        )
    },
    imports: []
)
