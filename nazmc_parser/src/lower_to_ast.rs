use crate::*;
use bumpalo::Bump;
use nazmc_data_pool::PoolIdx;
use std::collections::HashMap;

pub fn lower_to_ast(
    arena: &mut Bump,
    File {
        content: ZeroOrMany { items, .. },
    }: File,
) {
    for item in items {
        match item.unwrap() {
            FileItem::WithVisModifier(ItemWithVisibility { visibility, item }) => todo!(),
            FileItem::WithoutModifier(item) => todo!(),
        }
    }

    fn lower_item(arena: &mut Bump, item: Item, visibility: u64) {}

    fn lower_struct(arena: &mut Bump, struct_: Struct) {}

    fn lower_fn(arena: &mut Bump, fn_: Fn) {}

    fn lower_lambda_expr(arena: &mut Bump, lambda_expr: LambdaExpr) {}

    fn lower_stm(arena: &mut Bump, stm: Stm) {}

    fn lower_expr(arena: &mut Bump, expr: Expr) {}
}
