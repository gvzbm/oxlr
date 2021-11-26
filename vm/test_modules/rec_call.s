Module(
    path: Path([Symbol("rec_call")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("fib"): (
            // (int, int, int) -> int
            FunctionSignature(args: [
                    (Int(width: 64, signed: false), Symbol("n"))
                ],
                return_type: Int(width: 64, signed: false)
            ),
            // this code is sloppy and actually computes fib(n+2), oh well
            FnBody(
                max_registers: 7,
                blocks: [
                    BasicBlock(
                        instrs: [
                            BinaryOp(Eq,  Register(1), Reg(Register(0)), LiteralInt(Integer(width: 64, signed: false, data: 0))),
                            Br(cond: Reg(Register(1)), if_true: 1, if_false: 2)
                        ],
                        next_block: 999
                    ),

                    BasicBlock(
                        instrs: [
                            Return(LiteralInt(Integer(width: 64, signed: false, data: 1)))
                        ],
                        next_block: 999
                    ),

                    BasicBlock(
                        instrs: [
                            BinaryOp(Sub, Register(2), Reg(Register(0)), LiteralInt(Integer(width: 64, signed: false, data: 1))),
                            BinaryOp(Sub, Register(3), Reg(Register(0)), LiteralInt(Integer(width: 64, signed: false, data: 2))),
                            Call(Register(4), Path([Symbol("rec_call"), Symbol("fib")]), [ Reg(Register(2)) ]),
                            Call(Register(5), Path([Symbol("rec_call"), Symbol("fib")]), [ Reg(Register(3)) ]),
                            BinaryOp(Add, Register(6), Reg(Register(4)), Reg(Register(5))),
                            Return(Reg(Register(6)))
                        ],
                        next_block: 999
                    )
                ]
            )
        ),
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 2,
                blocks: [
                    BasicBlock(
                        instrs: [
                            Call(Register(0), Path([Symbol("rec_call"), Symbol("fib")]), [ LiteralInt(Integer(width: 64, signed: false, data: 10)) ]),
                            BinaryOp(Sub, Register(1), LiteralInt(Integer(width: 64, signed: false, data: 144)), Reg(Register(0))),
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
