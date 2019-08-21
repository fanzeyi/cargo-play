//# regex-syntax = "*"

use regex_syntax::hir::{self, Hir};
use regex_syntax::Parser;

fn main() {
    let hir = Parser::new().parse("a|b").unwrap();
    assert_eq!(
        hir,
        Hir::alternation(vec![
            Hir::literal(hir::Literal::Unicode('a')),
            Hir::literal(hir::Literal::Unicode('b')),
        ])
    );
}
