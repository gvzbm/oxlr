Module(
    path: Path([Symbol("stack_array_alloc")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 6,
                blocks: [
                    BasicBlock(
                        instrs: [
                            StackAllocArray(Register(0), Int(width: 64, signed: false), LiteralInt(Integer(width: 64, signed: false, data: 3))),
                            StoreIndex(Register(0),
                                LiteralInt(Integer(width: 64, signed: false, data: 0)),
                                LiteralInt(Integer(width: 64, signed: false,  data: 15))),
                            StoreIndex(Register(0),
                                LiteralInt(Integer(width: 64, signed: false, data: 2)),
                                LiteralInt(Integer(width: 64, signed: false,  data: 9))),
                            StoreIndex(Register(0),
                                LiteralInt(Integer(width: 64, signed: false, data: 1)),
                                LiteralInt(Integer(width: 64, signed: false,  data: 6))),
                            LoadIndex(Register(1), Register(0), LiteralInt(Integer(width: 64, signed: false, data: 1))),
                            LoadIndex(Register(2), Register(0), LiteralInt(Integer(width: 64, signed: false, data: 2))),
                            LoadIndex(Register(3), Register(0), LiteralInt(Integer(width: 64, signed: false, data: 0))),
                            BinaryOp(Add, Register(4), Reg(Register(1)), Reg(Register(2))),
                            BinaryOp(Sub, Register(5), Reg(Register(4)), Reg(Register(3))),
                            Return(Reg(Register(5)))
                        ],
                        next_block: 99
                    )
                ]
            )
        )
    },
    imports: []
)
