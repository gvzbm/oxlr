Module(
    path: Path([Symbol("basic_struct")]),
    version: "0.0.1",
    types: {
        Symbol("foo"): Product(
            parameters: [],
            fields: [
                (Symbol("a"), Int(signed: false, width: 64)),
                (Symbol("b"), Int(signed: false, width: 64)),
                (Symbol("c"), Bool),
            ]
        )
    },
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
                            Alloc(Register(0), User(Path([Symbol("basic_struct"), Symbol("foo")]), None)),
                            StoreField(LiteralInt(Integer(width: 64, signed: false, data: 3)), Register(0), Symbol("a")),
                            StoreField(LiteralInt(Integer(width: 64, signed: false, data: 9)), Register(0), Symbol("b")),
                            StoreField(LiteralBool(true), Register(0), Symbol("c")),
                            LoadField(Register(1), Register(0), Symbol("a")),
                            LoadField(Register(2), Register(0), Symbol("b")),
                            LoadField(Register(3), Register(0), Symbol("c")),
                            BinaryOp(Add, Register(4), Reg(Register(1)), Reg(Register(2))),
                            BinaryOp(Sub, Register(5), Reg(Register(4)), LiteralInt(Integer(width: 64, signed: false, data: 12))),
                            Return(/* LiteralInt(Integer(width: 64, signed: false, data: 0)) */ Reg(Register(5)))
                        ],
                        next_block: 0
                    )
                ]
            )
        )
    },
    imports: []
)
