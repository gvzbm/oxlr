Module(
    path: Path([Symbol("ref_to_inner_field")]),
    version: "0.0.1",
    types: {
        Symbol("foo"): Product(
            parameters: [],
            fields: [
                (Symbol("a"), Int(signed: false, width: 64)),
                (Symbol("b"), Int(signed: false, width: 64)),
            ]
        )
    },
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
                            Alloc(Register(0), User(Path([Symbol("ref_to_inner_field"), Symbol("foo")]), None)),
                            StoreField(LiteralInt(Integer(width: 64, signed: false, data: 3)), Register(0), Symbol("a")),
                            StoreField(LiteralInt(Integer(width: 64, signed: false, data: 9)), Register(0), Symbol("b")),
                            RefField(Register(1), Register(0), Symbol("b")),
                            Call(Register(2), Path([Symbol("ref_to_inner_field"), Symbol("incr")]),
                            [Reg(Register(1))]),
                            LoadField(Register(3), Register(0), Symbol("b")),
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
