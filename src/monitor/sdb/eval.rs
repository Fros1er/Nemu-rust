use crate::isa::riscv64::vaddr::MemOperationSize::DWORD;
use crate::isa::riscv64::vaddr::VAddr;
use crate::isa::Isa;
use crate::Emulator;
use chumsky::prelude::*;
use chumsky::Parser;

/*
<expr> ::= <decimal-number>
  | <hexadecimal-number>    # 以"0x"开头
  | <reg_name>              # 以"$"开头
  | "(" <expr> ")"
  | <expr> "+" <expr>
  | <expr> "-" <expr>
  | <expr> "*" <expr>
  | <expr> "/" <expr>
  | <expr> "==" <expr>
  | <expr> "!=" <expr>
  | <expr> "&&" <expr>
  | "*" <expr>              # 指针解引用
 */

#[derive(Debug)]
pub(crate) enum Expr {
    Num(i64),
    Reg(String),
    Neg(Box<Expr>),
    Deref(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    EQ(Box<Expr>, Box<Expr>),
    NEQ(Box<Expr>, Box<Expr>),
    AND(Box<Expr>, Box<Expr>),
}

fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    recursive(|expr| {
        let hex = just("0x")
            .ignore_then(text::int(16))
            .map(|s: String| Expr::Num(i64::from_str_radix(s.as_str(), 16).unwrap()));

        let int = text::int(10).map(|s: String| Expr::Num(s.parse().unwrap()));

        let reg = just('$')
            .ignore_then(text::ident())
            .map(|s: String| Expr::Reg(s))
            .padded();

        let atom = hex
            .or(int)
            .or(reg)
            .or(expr.delimited_by(just('('), just(')')))
            .padded();

        let op = |c| just(c).padded();
        let op_str = |s| just(s).padded();

        let neg = op('-')
            .repeated()
            .then(atom.clone())
            .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

        let deref = op('*')
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Expr::Deref(Box::new(rhs)));

        let unary = neg.or(deref);

        let product = unary
            .clone()
            .then(
                op('*')
                    .to(Expr::Mul as fn(_, _) -> _)
                    .or(op('/').to(Expr::Div as fn(_, _) -> _))
                    .then(unary)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let sum = product
            .clone()
            .then(
                op('+')
                    .to(Expr::Add as fn(_, _) -> _)
                    .or(op('-').to(Expr::Sub as fn(_, _) -> _))
                    .then(product)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let comp = sum
            .clone()
            .then(
                op_str("==")
                    .to(Expr::EQ as fn(_, _) -> _)
                    .or(op_str("!=").to(Expr::NEQ as fn(_, _) -> _))
                    .then(sum)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let and = comp
            .clone()
            .then(
                op_str("&&")
                    .to(Expr::AND as fn(_, _) -> _)
                    .then(comp)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));
        and
    })
    .then_ignore(end())
}

pub fn eval_expr<T: Isa>(expr: &Expr, emulator: &mut Emulator<T>) -> Result<i64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Reg(x) => emulator
            .cpu
            .isa_get_reg_by_name(x.as_str())
            .map(|v| v as i64),
        Expr::Deref(a) => {
            let addr = eval_expr(a, emulator)? as u64;
            emulator
                .cpu
                .read_vaddr(&VAddr::new(addr), DWORD)
                .map(|x| x as i64)
        }
        Expr::Neg(a) => Ok(-eval_expr(a, emulator)?),
        Expr::Add(a, b) => Ok(eval_expr(a, emulator)? + eval_expr(b, emulator)?),
        Expr::Sub(a, b) => Ok(eval_expr(a, emulator)? - eval_expr(b, emulator)?),
        Expr::Mul(a, b) => Ok(eval_expr(a, emulator)? * eval_expr(b, emulator)?),
        Expr::Div(a, b) => Ok(eval_expr(a, emulator)? / eval_expr(b, emulator)?),
        Expr::EQ(a, b) => Ok(i64::from(
            eval_expr(a, emulator)? == eval_expr(b, emulator)?,
        )),
        Expr::NEQ(a, b) => Ok(i64::from(
            eval_expr(a, emulator)? != eval_expr(b, emulator)?,
        )),
        Expr::AND(a, b) => Ok(i64::from(
            eval_expr(a, emulator)? != 0 && eval_expr(b, emulator)? != 0,
        )),
    }
}

pub fn parse(expr: &str) -> Result<Expr, String> {
    parser().parse(expr).map_err(|err| {
        err.iter()
            .fold(String::new(), |err_str, it| err_str + &it.to_string())
    })
}

pub fn eval<T: Isa>(expr: &str, emulator: &mut Emulator<T>) -> Result<i64, String> {
    eval_expr(&parse(expr)?, emulator)
}

// #[cfg(test)]
// mod tests {
//     use crate::monitor::sdb::eval::{eval, eval_expr, parser};
//     use crate::utils::tests::fake_emulator;
//     use chumsky::Parser;
//     //
//     //     #[test]
//     //     fn mem_test() {
//     // let emulator = fake_emulator();
//     // let test_addr: VAddr = (CONFIG_MBASE + 8).into();
//     // emulator.
//     //     .memory
//     //     .borrow_mut()
//     //     .write(&test_addr, 114514, QWORD);
//     // let exp = "*0x80000008".to_string();
//     // assert_eq!(
//     //     eval_expr(&parser().parse(exp).unwrap(), &emulator).unwrap(),
//     //     114514
//     // );
//     // }
//
//     #[test]
//     fn calc_test() {
//         let exp = "100 * 0xa - ((1 + 1) + 2) -- 1".to_string();
//         let mut emulator = fake_emulator();
//         assert_eq!(
//             eval_expr(&parser().parse(exp).unwrap(), &mut emulator).unwrap(),
//             997
//         );
//     }
//
//     #[test]
//     fn comp_test() {
//         let exp = "1 * 2 == 3 - 1 && 0x10 != 10".to_string();
//         let mut emulator = fake_emulator();
//         assert_eq!(
//             eval_expr(&parser().parse(exp).unwrap(), &mut emulator).unwrap(),
//             1
//         );
//     }
//
//     #[test]
//     fn err_test() {
//         let exp = "aaa * bbb".to_string();
//         let mut emulator = fake_emulator();
//         match eval(&exp, &mut emulator) {
//             Ok(_) => assert!(false),
//             Err(err) => println!("ERR: {}", err),
//         }
//     }
// }
