Module(
    path: Path([Symbol("simple_alloc")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("subtract 22"): (
            FunctionSignature(args: [(Ref(Int(width: 64, signed: false)), Symbol("x"))], return_type: Unit),
            FnBody(
                max_registers: 3,
                blocks: [
                    BasicBlock(
                        instrs: [
                            LoadRef(Register(1), Register(0)),
                            BinaryOp(Sub, Register(2), Reg(Register(1)), LiteralInt(Integer(width: 64, signed: false, data: 22))),
                            StoreRef(Register(0), Reg(Register(2))),
                            Return(LiteralUnit)
                        ],
                        next_block: 99
                    )
                ]
            )
        ),
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 3,
                blocks: [
                    BasicBlock(
                        instrs: [
                            Alloc(Register(0), Int(width: 64, signed: false)),
                            StoreRef(Register(0), LiteralInt(Integer(width: 64, signed: false, data: 22))),
                            Call(Register(1), Path([Symbol("simple_alloc"), Symbol("subtract 22")]), [Reg(Register(0))]),
                            LoadRef(Register(2), Register(0)),
                            Return(Reg(Register(2)))
                        ],
                        next_block: 99
                    )
                ]
            )
        )
    },
    imports: []
)
