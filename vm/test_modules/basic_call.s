Module(
    path: Path([Symbol("basic_call")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("muladd"): (
            // (int, int, int) -> int
            FunctionSignature(args: [
                    (Int(width: 32, signed: true), Symbol("a")),
                    (Int(width: 32, signed: true), Symbol("b")),
                    (Int(width: 32, signed: true), Symbol("c"))
                ],
                return_type: Int(width: 32, signed: true)
            ),
            FnBody(
                max_registers: 5,
                blocks: [
                    BasicBlock(
                        instrs: [
                            BinaryOp(Add, Register(3), Reg(Register(0)), Reg(Register(1))),
                            BinaryOp(Mul, Register(4), Reg(Register(2)), Reg(Register(3))),
                            Return(Reg(Register(4)))
                        ],
                        next_block: 0
                    )
                ]
            )
        ),
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 1,
                blocks: [
                    BasicBlock(
                        instrs: [
                            Call(Register(0), Path([Symbol("basic_call"), Symbol("muladd")]),
                                [ LiteralInt(Integer(width: 32, signed: true, data: 3)),
                                LiteralInt(Integer(width: 32, signed: true, data: 5)),
                                LiteralInt(Integer(width: 32, signed: true, data: 0)) ]),
                            Return(Reg(Register(0)))
                        ],
                        next_block: 0
                    )
                ]
            )
        )
    },
    imports: []
)
