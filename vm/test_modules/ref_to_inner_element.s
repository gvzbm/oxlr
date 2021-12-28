Module(
    path: Path([Symbol("ref_to_inner_element")]),
    version: "0.0.1",
    types: {},
    interfaces: {},
    implementations: {},
    functions: {
        Symbol("incr"): (
            FunctionSignature(args: [(Ref(Int(width: 64, signed: false)), Symbol("p"))], return_type: Unit),
            FnBody(
                max_registers: 3,
                blocks: [
                    BasicBlock(
                        instrs: [
                            LoadRef(Register(1), Register(0)),
                            BinaryOp(Add, Register(2), Reg(Register(1)),
                                LiteralInt(Integer(width: 64, signed: false, data: 1))),
                            StoreRef(Register(0), Reg(Register(2))),
                            Return(LiteralUnit)
                        ],
                        next_block: 999
                    )
                ]
            )
        ),
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 5,
                blocks: [
                    BasicBlock(
                        instrs: [
                            AllocArray(Register(0), Int(width: 64, signed: false),
                                LiteralInt(Integer(width: 64, signed: false, data: 3))),
                            StoreIndex(Register(0),
                                LiteralInt(Integer(width: 64, signed: false, data: 0)),
                                LiteralInt(Integer(width: 64, signed: false,  data: 15))),
                            StoreIndex(Register(0),
                                LiteralInt(Integer(width: 64, signed: false, data: 2)),
                                LiteralInt(Integer(width: 64, signed: false,  data: 9))),
                            StoreIndex(Register(0),
                                LiteralInt(Integer(width: 64, signed: false, data: 1)),
                                LiteralInt(Integer(width: 64, signed: false,  data: 6))),
                            RefIndex(Register(1), Register(0), LiteralInt(Integer(width: 64, signed: false, data: 1))),
                            Call(Register(2), Path([Symbol("ref_to_inner_element"), Symbol("incr")]),
                            [Reg(Register(1))]),
                            LoadIndex(Register(3), Register(0), LiteralInt(Integer(width: 64,
                                signed: false, data: 1))),
                            BinaryOp(Sub, Register(4), Reg(Register(3)),
                                LiteralInt(Integer(width: 64, signed: false, data: 10))),
                            Return(Reg(Register(4)))
                       ],
                        next_block: 0
                    )
                ]
            )
        )
    },
    imports: []
)
