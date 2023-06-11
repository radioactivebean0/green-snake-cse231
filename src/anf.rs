use core::panic;
use std::collections::{HashSet};
use im::HashMap;

use crate::syntax::{Expr, FunDecl, Symbol, Prog, Op1, Op2};
pub enum FlatVal {
    Num(i64),
    True,
    False,
    Var(Symbol),
}

pub enum FlatOp {
    Add1(Box<FlatVal>),
    Sub1(Box<FlatVal>),
    Plus(Box<FlatVal>, Box<FlatVal>),
    Minus(Box<FlatVal>, Box<FlatVal>),
    Times(Box<FlatVal>, Box<FlatVal>),
    Divide(Box<FlatVal>, Box<FlatVal>),
    Eq(Box<FlatVal>, Box<FlatVal>),
    Gt(Box<FlatVal>, Box<FlatVal>),
    Ge(Box<FlatVal>, Box<FlatVal>),
    Lt(Box<FlatVal>, Box<FlatVal>),
    Le(Box<FlatVal>, Box<FlatVal>),

    IsNum(Box<FlatVal>),
    IsBool(Box<FlatVal>),
    IsVec(Box<FlatVal>),

    Print(Box<FlatVal>),
    Set(Symbol, Box<FlatVal>),

    Call(Symbol, Vec<FlatVal>),

    MakeVec(Box<FlatVal>, Box<FlatVal>),
    Vec(Vec<FlatVal>),
    VecSet(Box<FlatVal>, Box<FlatVal>, Box<FlatVal>),
    VecGet(Box<FlatVal>, Box<FlatVal>),
    VecLen(Box<FlatVal>),

    Break(Box<FlatVal>),
    Loop(Box<FlatBlock>),

    If(Box<FlatVal>, Box<FlatBlock>, Box<FlatBlock>),

    Val(Box<FlatVal>),

    Input,
    Nil,
    PrintStack,
    Gc,
}

pub enum FlatBlock {
    Let(Symbol, Box<FlatOp>, Box<FlatBlock>),
    Block(Vec<FlatBlock>),
    Op(Box<FlatOp>),
}

pub struct FlatProgram {
    pub defs: Vec<FlatDefinition>,
    pub main: FlatBlock,
}

pub struct FlatDefinition {
    pub name: Symbol,
    pub args: Vec<Symbol>,
    pub body: FlatBlock,
}

fn new_label(l: &mut i32, s: &str) -> Symbol {
    let current = *l;
    *l += 1;
    Symbol::new(format!("{s}_{current}"))
}

fn get_uniq_name(s: Symbol, idx: u32) -> Symbol{
    Symbol::new (format!("uq_{s}_{idx}"))
}

fn anf_val(e: &Expr, i: &mut i32, in_main: bool, bound_vars: &HashMap<Symbol, u32>) -> (FlatVal, Vec<(Symbol, FlatOp)>) {
    match e {
        Expr::Number(n) => (FlatVal::Num(*n), vec![]),
        Expr::Var(s) => (FlatVal::Var(get_uniq_name(s.clone(), *bound_vars.get(s).unwrap())), vec![]),
        Expr::Boolean(b) if *b==true => (FlatVal::True, vec![]),
        Expr::Boolean(_) => (FlatVal::False,vec![]),
        _ => {
            let (op, mut binds) = anf_expr(e, i, in_main, bound_vars);
            let tmp = new_label(i, "%t");
            binds.push((tmp.clone(), op));
            (FlatVal::Var(tmp), binds)
        }
    }
}

fn anf_op1(op: &Op1, e: &Expr, i: &mut i32, in_main: bool, bound_vars: &HashMap<Symbol, u32>) -> (FlatOp, Vec<(Symbol, FlatOp)>){
    let (e, binds) = anf_val(e, i, in_main, bound_vars);
    match op {
        Op1::Add1 => (FlatOp::Add1(Box::new(e)), binds),
        Op1::Sub1 => (FlatOp::Sub1(Box::new(e)), binds),
        Op1::IsNum => (FlatOp::IsNum(Box::new(e)), binds),
        Op1::IsBool => (FlatOp::IsBool(Box::new(e)), binds),
        Op1::IsVec => (FlatOp::IsVec(Box::new(e)), binds),
        Op1::Print => (FlatOp::Print(Box::new(e)), binds),
    }
}

fn anf_op2(op: &Op2, e1: &Expr, e2: &Expr, i: &mut i32, in_main: bool, bound_vars: &HashMap<Symbol, u32>) -> (FlatOp, Vec<(Symbol, FlatOp)>) {
    let (e1, mut binds1) = anf_val(e1, i, in_main, bound_vars);
    let (e2, mut binds2) = anf_val(e2, i, in_main, bound_vars);
    binds1.append(&mut binds2);
    match op {
        Op2::Plus => (FlatOp::Plus(Box::new(e1), Box::new(e2)), binds1),
        Op2::Minus => (FlatOp::Minus(Box::new(e1), Box::new(e2)), binds1),
        Op2::Times => (FlatOp::Times(Box::new(e1), Box::new(e2)), binds1),
        Op2::Divide => (FlatOp::Divide(Box::new(e1), Box::new(e2)), binds1),
        Op2::Equal => (FlatOp::Eq(Box::new(e1), Box::new(e2)), binds1),
        Op2::Greater => (FlatOp::Gt(Box::new(e1), Box::new(e2)), binds1),
        Op2::GreaterEqual => (FlatOp::Ge(Box::new(e1), Box::new(e2)), binds1),
        Op2::Less => (FlatOp::Lt(Box::new(e1), Box::new(e2)), binds1),
        Op2::LessEqual => (FlatOp::Le(Box::new(e1), Box::new(e2)), binds1),
    }
    
}

fn anf_expr(e: &Expr, i: &mut i32, in_main: bool, bound_vars: &HashMap<Symbol, u32>) -> (FlatOp, Vec<(Symbol, FlatOp)>) {
    match e {
        Expr::Number(n) => (FlatOp::Val(Box::new(FlatVal::Num(*n))), vec![]),
        Expr::Boolean(b) if *b==true => (FlatOp::Val(Box::new(FlatVal::True)), vec![]),
        Expr::Boolean(_) => (FlatOp::Val(Box::new(FlatVal::False)), vec![]),
        Expr::Var(s) => (FlatOp::Val(Box::new(FlatVal::Var(get_uniq_name(*s, *bound_vars.get(s).unwrap())))), vec![]),
        Expr::Let(binds, body) => {
            let mut anfbinds = vec![];
            let mut index = 0;
            let mut seen:HashSet<Symbol> = HashSet::new();
            let mut bind_vars = bound_vars.clone();
            for (s, e) in binds.into_iter() {
                if !seen.insert(*s) {
                    raise_duplicate_binding(*s);
                }
                let e_vars = bind_vars.clone();
                let uniq_s;
                match bind_vars.clone().get(s) {
                    Some(idx) => {
                        bind_vars = bind_vars.update(s.clone(), idx+1);
                        uniq_s = get_uniq_name(s.clone(), idx+1);
                    }
                    None => {
                        bind_vars = bind_vars.update(s.clone(), 0);
                        uniq_s = get_uniq_name(s.clone(), 0);
                    }
                }
                if index == binds.len() - 1 {
                    let (v, mut vbinds) = anf_expr(e, i, in_main, &e_vars);
                    let (body, mut bbinds) = anf_expr(body, i, in_main, &bind_vars);
                    anfbinds.append(&mut vbinds);
                    anfbinds.push((uniq_s, v));
                    anfbinds.append(&mut bbinds);
                    return (body, anfbinds);
                }
                index += 1;
                let (v, mut vbinds) = anf_expr(e, i, in_main, &e_vars);
                anfbinds.append(&mut vbinds);
                anfbinds.push((uniq_s, v));
            }
            panic!("empty let")
        },
        Expr::UnOp(op, e) => anf_op1(op, e, i, in_main, bound_vars),
        Expr::BinOp(op, e1, e2) => anf_op2(op, e1, e2, i, in_main, bound_vars),
        Expr::If(e1, e2, e3) => {
            let (e1, binds1) = anf_val(e1, i, in_main, bound_vars);
            let e2 = anf_block(e2, i, in_main, bound_vars);
            let e3 = anf_block(e3, i, in_main, bound_vars);
            (FlatOp::If(Box::new(e1), Box::new(e2), Box::new(e3)), binds1)
        },
        Expr::Loop(e) => (FlatOp::Loop(Box::new(anf_block(e, i, in_main, bound_vars))), vec![]),
        Expr::Break(e) => {
            let (e, binds) = anf_val(e, i, in_main, bound_vars);
            (FlatOp::Break(Box::new(e)), binds)
        },
        Expr::Set(x, e) => {
            let (e, binds) = anf_val(e, i, in_main, bound_vars);
            (FlatOp::Set(get_uniq_name(x.clone(), *bound_vars.get(x).unwrap()), Box::new(e)), binds)
        },
        Expr::MakeVec(cnt, val) => {
            let (c, mut binds1) = anf_val(cnt, i, in_main, bound_vars);
            let (v, mut binds2) = anf_val(val, i, in_main, bound_vars);
            binds1.append(&mut binds2);
            (FlatOp::MakeVec(Box::new(c), Box::new(v)), binds1)
        },
        Expr::Vec(es) => {
            let mut binds = vec![];
            let mut flat_vec = vec![];
            for e in es {
                let (flate, mut tmpbind) = anf_val(e, i, in_main, bound_vars);
                binds.append(&mut tmpbind);
                flat_vec.push(flate);
            }
            (FlatOp::Vec(flat_vec), binds)
        },
        Expr::VecSet(vec, ind, val) => {
            let (vc, mut binds1) = anf_val(vec, i, in_main, bound_vars);
            let (id, mut binds2) = anf_val(ind, i, in_main, bound_vars);
            let (vl, mut binds3) = anf_val(val, i, in_main, bound_vars);
            binds1.append(&mut binds2);
            binds1.append(&mut binds3);
            (FlatOp::VecSet(Box::new(vc), Box::new(id), Box::new(vl)), binds1)
        },
        Expr::VecGet(vec, ind) => {
            let (vc, mut binds1) = anf_val(vec, i, in_main, bound_vars);
            let (id, mut binds2) = anf_val(ind, i, in_main, bound_vars);
            binds1.append(&mut binds2);
            (FlatOp::VecGet(Box::new(vc), Box::new(id)), binds1)
        },
        Expr::VecLen(vec) => {
            let (vc, binds1) = anf_val(vec, i, in_main, bound_vars);
            (FlatOp::VecLen(Box::new(vc)), binds1)
        },
        Expr::Block(vec) => {
            let mut binds = vec![];
            let mut index = 0;
            for e in vec {
                if index == vec.len() - 1 {
                    let (e, mut ebinds) = anf_expr(e, i, in_main, bound_vars);
                    binds.append(&mut ebinds);
                    return (e, binds);
                }
                index += 1;
                let (e, mut ebinds) = anf_expr(e, i, in_main, bound_vars);
                let tmp = new_label(i, "%block_unused_");
                binds.append(&mut ebinds);
                binds.push((tmp.clone(), e));
            }
            panic!("Empty block")
        },
        Expr::Call(name, args) => {
            let mut binds = vec![];
            let mut aargs = vec![];
            for arg in args {
                let (e, mut ebind) = anf_val(arg, i, in_main, bound_vars);
                binds.append(&mut ebind);
                aargs.push(e);
            }
            (FlatOp::Call(name.clone(), aargs), binds)
        },
        Expr::Input if !in_main => panic!("cannot use input inside funciton definition"),
        Expr::Input => (FlatOp::Input, vec![]),
        Expr::Nil => (FlatOp::Nil, vec![]),
        Expr::PrintStack => (FlatOp::PrintStack, vec![]),
        Expr::Gc => (FlatOp::Gc, vec![]),
    }
}

fn anf_block(e: &Expr, i: &mut i32, in_main: bool, bound_vars: &HashMap<Symbol, u32>) -> FlatBlock {
    match e {
        Expr::Let(binds, body) => {
            let mut body_vars = bound_vars.clone();
            let mut seen:HashSet<Symbol> = HashSet::new();
            for (s, _) in binds.into_iter() { // reserve names in body
                if !seen.insert(*s) {
                    raise_duplicate_binding(*s);
                }
                match body_vars.get(s) {
                    Some(idx) => body_vars = body_vars.update(s.clone(), idx+1),
                    None => body_vars = body_vars.update(s.clone(), 0),
                }
            }
            let mut body = anf_block(body, i, in_main, &body_vars);
            let mut bind_vars = body_vars.clone();
            for (s, e) in binds.into_iter().rev() {
                let uniq_s;
                match bind_vars.clone().get(s) { // update the bound vars
                    Some(idx) => {
                        if *idx == (0 as u32) {
                            bind_vars = bind_vars.without(s);
                            uniq_s = get_uniq_name(s.clone(), 0);
                        } else {
                            bind_vars = bind_vars.update(s.clone(), idx-1);
                            uniq_s = get_uniq_name(s.clone(), *idx);
                        }
                    }
                    None => panic!("shouldn't happen")
                }
                let e_vars = bind_vars.clone();
                let (v, binds1) = anf_expr(e, i, in_main, &e_vars);
                body = FlatBlock::Let(uniq_s, Box::new(v), Box::new(body));

                for (name, val) in binds1.into_iter().rev() {
                    body = FlatBlock::Let(name, Box::new(val), Box::new(body));
                }
            }
            return body;
        }
        Expr::Block(vec) => {
            let mut blocks = vec![];
            for e in vec {
                let e = anf_block(e, i, in_main, bound_vars);
                blocks.push(e);
            }
            FlatBlock::Block(blocks)
        }
        _ => {
            let (op, binds) = anf_expr(e, i, in_main, bound_vars);
            let mut block = FlatBlock::Op(Box::new(op));
            for (x, v) in binds.into_iter().rev() {
                block = FlatBlock::Let(x, Box::new(v), Box::new(block));
            }
            block
        }
    }
}

fn anf_definition(e: &FunDecl) -> FlatDefinition {
    let mut i = 0;
    let mut newp = vec![];
    let mut var_binds:HashMap<Symbol, u32> = HashMap::new();
    for p in &e.params {
        var_binds = var_binds.update(*p, 0);
        newp.push(get_uniq_name(p.clone(), 0));
    }
    FlatDefinition { name: e.name, args: newp, body: anf_block(&e.body, &mut i, false, &var_binds) }
}

pub fn anf_program(p: &Prog) -> FlatProgram {
    let mut defs = vec![];
    for d in &p.funs {
        defs.push(anf_definition(d));
    }
    check_duplicate_functions(&defs);
    let mut i = 0;
    let main = anf_block(&p.main, &mut i, true, &HashMap::new());
    FlatProgram { main, defs }
}

fn raise_duplicate_binding(id: Symbol) {
    panic!("duplicate binding {id}");
}

fn check_duplicate_functions(defs: &Vec<FlatDefinition>) {
    let mut seen: HashSet<Symbol> = HashSet::new();
    for d in defs {
        if !seen.insert(d.name) {
            panic!("duplicate function name {}",d.name);
        }
    }
}

/// Takes a program and returns a string of the program as an s-expression; uses
/// helper functions expr_to_string and val_to_string
fn block_to_string(e: &FlatBlock) -> String {
    match e {
        FlatBlock::Op(op) => op_to_string(op),
        FlatBlock::Let(x, v, body) => {
            format!("(let {} {} {})", x, op_to_string(v), block_to_string(body))
        }
        FlatBlock::Block(vec) => {
            let mut s = String::from("(block");
            for e in vec {
                s.push_str(&format!(" {}", block_to_string(e)));
            }
            s.push_str(")");
            s
        }
    }
}

fn flatdefinition_to_string(e: &FlatDefinition) -> String {
    let mut s = format!("(fun ({}",e.name);
    for arg in &e.args {
        s = s+ " " + &arg.to_string();
    }
    return format!("{}) {})", s,block_to_string(&e.body));
}

pub fn flatprogram_to_string(e: &FlatProgram) -> String {
    let mut s = String::from("");
    for d in &e.defs {
        s.push_str(&format!("{}\n\n", flatdefinition_to_string(d)));
    }
    s.push_str(&format!("{}", block_to_string(&e.main)));
    s
}

fn op_to_string(e: &FlatOp) -> String {
    match e {
        FlatOp::Add1(e) => format!("(add1 {})", val_to_string(e)),
        FlatOp::Sub1(e) => format!("(sub1 {})", val_to_string(e)),
        FlatOp::Plus(e1, e2) => format!("(+ {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::Minus(e1, e2) => format!("(- {} {})", val_to_string(e1), val_to_string(e2)),
        //FlatOp::Pair(e1, e2) => format!("(pair {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::Print(e) => format!("(print {})", val_to_string(e)),
        // FlatOp::SetFst(e1, e2) => format!("(set-fst! {} {})", val_to_string(e1), val_to_string(e2)),
        // FlatOp::SetSnd(e1, e2) => format!("(set-snd! {} {})", val_to_string(e1), val_to_string(e2)),
        // FlatOp::Fst(e) => format!("(fst {})", val_to_string(e)),
        // FlatOp::Snd(e) => format!("(snd {})", val_to_string(e)),
        FlatOp::Set(x, e) => format!("(set! {} {})", x, val_to_string(e)),
        FlatOp::Break(e) => format!("(break {})", val_to_string(e)),
        // FlatOp::Call1(f, e) => format!("(call1 {} {})", f, val_to_string(e)),
        // FlatOp::Call2(f, e1, e2) => {
        //     format!("(call2 {} {} {})", f, val_to_string(e1), val_to_string(e2))
        // }
        FlatOp::Eq(e1, e2) => format!("(= {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::Lt(e1, e2) => format!("(< {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::If(e1, e2, e3) => format!(
            "(if {} {} {})",
            val_to_string(e1),
            block_to_string(e2),
            block_to_string(e3)
        ),
        FlatOp::Loop(e) => format!("(loop {})", block_to_string(e)),
        FlatOp::Val(v) => val_to_string(v),
        FlatOp::Times(e1, e2) => format!("(* {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::Divide(e1, e2) => format!("(/ {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::Gt(e1, e2) => format!("(> {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::Ge(e1, e2) => format!("(>= {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::Le(e1, e2) => format!("(<= {} {})", val_to_string(e1), val_to_string(e2)),
        FlatOp::IsNum(v) => format!("(isNum {})", val_to_string(v)),
        FlatOp::IsBool(v) =>format!("(isBool {})", val_to_string(v)),
        FlatOp::IsVec(v) => format!("(isVec {})", val_to_string(v)),
        FlatOp::Call(nm, args) => {
            let mut s = format!("(call {}",nm).to_string();
            for arg in args {
                s = format!("{} {}", s, val_to_string(arg));
            }
            return format!("{})",s);
        },
        FlatOp::MakeVec(sz, v) => format!("(make-vec {} {})", val_to_string(sz), val_to_string(v)),
        FlatOp::Vec(es) => {
            let mut s = "(vec".to_string();
            for elem in es {
                s = format!("{} {}", s, val_to_string(elem));
            }
            return format!("{})",s);
        },
        FlatOp::VecSet(v, ix, v2) => format!("(vec-set {} {} {})", val_to_string(v), val_to_string(ix), val_to_string(v2)),
        FlatOp::VecGet(v, ix) => format!("(vec-get {} {})", val_to_string(v), val_to_string(ix)),
        FlatOp::VecLen(v) => format!("(veclen {})", val_to_string(v)),
        FlatOp::Input => "input".to_string(),
        FlatOp::Nil => "nil".to_string(),
        FlatOp::PrintStack => "printstack".to_string(),
        FlatOp::Gc => "gc".to_string(),
    }
}

fn val_to_string(e: &FlatVal) -> String {
    match e {
        FlatVal::Num(n) => format!("{}", n),
        FlatVal::Var(x) => x.to_string(),
        FlatVal::True => String::from("true"),
        FlatVal::False => String::from("false"),
    }
}