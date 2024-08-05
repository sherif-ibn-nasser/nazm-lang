pub struct Token<'a>{
    pub val: &'a str,
    pub typ: TokenType,
}

pub enum TokenType {
    Bad,
    EOL,
    EOF,
    Space,
    Comment,
    Literal(LiteralTokenType),
    Id,
    Symbol,
    Keyword,
}

pub enum LiteralTokenType{
    String,
    Char,
    Bool,
    Num(NumTokenType),
}

pub enum NumTokenType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}