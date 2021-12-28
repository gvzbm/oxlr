Module(
    path: Path([Symbol("copy")]),
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
        Symbol("start"): (
            FunctionSignature(args: [], return_type: Int(width: 64, signed: false)),
            FnBody(
                max_registers: 7,
                blocks: [
                    BasicBlock(
                        instrs: [
                            Alloc(Register(0), User(Path([Symbol("copy"), Symbol("foo")]), None)),
                            StoreField(LiteralInt(Integer(width: 64, signed: false, data: 21)), Register(0), Symbol("a")),
                            StoreField(LiteralInt(Integer(width: 64, signed: false, data: 36)), Register(0), Symbol("b")),

                            CopyToStack(Register(1), Register(0)),
                            CopyToHeap(Register(2), Register(1)),

                            LoadField(Register(3), Register(2), Symbol("a")),
                            LoadField(Register(4), Register(2), Symbol("b")),
                            BinaryOp(Add, Register(5), Reg(Register(3)), Reg(Register(4))),
                            BinaryOp(Sub, Register(6), Reg(Register(5)), LiteralInt(Integer(width:
                            64, signed: false, data: 57))),
                            Return(Reg(Register(6)))
                        ],
                        next_block: 0
                    )
                ]
            )
        )
    },
    imports: []
)
