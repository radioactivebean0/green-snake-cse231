use crate::ir::*;

pub fn optimize_ir(prog: &Prog) -> Prog {
    let (mut new_prog, mut done) = fold_constants(prog);
    while !done {
        (new_prog, done) = fold_constants(&new_prog);
        //print!("{}", ir_to_string(&new_prog));
    }
    return new_prog;
}

fn fold_constants(prog: &Prog) -> (Prog, bool) {
    let mut done = true;
    let (new_defs, isdone) = fold_constants_defs(&prog.defs);
    let (new_body, tdone) = fold_constants_block(&prog.main);
    if !tdone || !isdone{
        done = false;
    }
    return (Prog{defs: new_defs, main: new_body},done)

}

fn fold_constants_defs(defs: &[Def]) -> (Vec<Def>, bool){
    let mut done = true;
    let mut new_defs = vec![];
    for def in defs {
        let (folded, isdone) = fold_constants_def(&def);
        if !isdone {
            done = false;
        }
        new_defs.push(folded);
    }
    return (new_defs, done);
}

fn fold_constants_def(def: &Def) -> (Def, bool) {
    let (folded, isdone) = fold_constants_block(&def.body);
    return (Def{name: def.name.clone(), args: def.args.clone(), body: folded}, isdone);
}

fn fold_constants_block(block: &Block) -> (Block, bool) {
    let (new_steps, isdone) = fold_constants_steps(&block.steps);
    return (Block{steps: new_steps}, isdone);
}

fn fold_constants_steps(steps: &[Step]) -> (Vec<Step>, bool) {
    let mut new_steps = vec![];
    let mut isdone = true;
    for step in steps {
        match step {
            Step::Set(x, e) => {
                let (newe, tdone) = fold_constants_expr(&e);
                if !tdone {
                    //println!("folded a step");
                    isdone = false;
                }
                new_steps.push(Step::Set(x.clone(), newe));
            },
            _ => new_steps.push(step.clone()),
        }
    }
    return (new_steps, isdone);
}

fn fold_constants_expr(e: &IRExpr) -> (IRExpr, bool) {
    match e {
        IRExpr::Add1(v) => {
            match v {
                Val::Num(n) => {
                    let res = n.checked_add(1);
                    match res {
                        Some(m) => (IRExpr::Val(Val::Num(m)), false),
                        None => (e.clone(), true), // just let overflow happen and get caught for now
                    }
                },
                _ => (e.clone(), true)
            }
        },
        IRExpr::Sub1(v) => {
            match v {
                Val::Num(n) => {
                    let res = n.checked_sub(1);
                    match res {
                        Some(m) => (IRExpr::Val(Val::Num(m)), false),
                        None => (e.clone(), true), // just let overflow happen and get caught for now
                    }
                },
                _ => (e.clone(), true)
            }
        }
        IRExpr::Plus(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    let res = n1.checked_add(*n2);
                    match res {
                        Some(m) => (IRExpr::Val(Val::Num(m)), false),
                        None => (e.clone(), true), // just let overflow happen and get caught for now
                    }
                }
                _ => (e.clone(), true)
            }
        },
        IRExpr::Minus(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    let res = n1.checked_sub(*n2);
                    match res {
                        Some(m) => (IRExpr::Val(Val::Num(m)), false),
                        None => (e.clone(), true), // just let overflow happen and get caught for now
                    }
                }
                _ => (e.clone(), true)
            }
        }
        IRExpr::Times(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    let res = n1.checked_mul(*n2);
                    match res {
                        Some(m) => (IRExpr::Val(Val::Num(m)), false),
                        None => (e.clone(), true), // just let overflow happen and get caught for now
                    }
                }
                _ => (e.clone(), true)
            }
        }
        IRExpr::Divide(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    let res = n1.checked_div(*n2);
                    match res {
                        Some(m) => (IRExpr::Val(Val::Num(m)), false),
                        None => (e.clone(), true), // just let overflow happen and get caught for now
                    }
                }
                _ => (e.clone(), true)
            }
        }
        IRExpr::Eq(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    if n1 == n2 {
                        return (IRExpr::Val(Val::True), false);
                    } else {
                        return (IRExpr::Val(Val::False), false);
                    }
                }
                (Val::True, Val::True) |
                (Val::False, Val::False) => (IRExpr::Val(Val::True), false),
                (Val::False, Val::True) |
                (Val::True, Val::False) => (IRExpr::Val(Val::False), false),
                _ => (e.clone(), true),
            }
        }
        IRExpr::Gt(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    if n1 > n2 {
                        return (IRExpr::Val(Val::True), false);
                    } else {
                        return (IRExpr::Val(Val::False), false);
                    }
                }
                _ => (e.clone(), true),
            }
        },
        IRExpr::Ge(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    if n1 >= n2 {
                        return (IRExpr::Val(Val::True), false);
                    } else {
                        return (IRExpr::Val(Val::False), false);
                    }
                }
                _ => (e.clone(), true),
            }
        }
        IRExpr::Lt(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    if n1 < n2 {
                        return (IRExpr::Val(Val::True), false);
                    } else {
                        return (IRExpr::Val(Val::False), false);
                    }
                }
                _ => (e.clone(), true),
            }
        }
        IRExpr::Le(v1, v2) => {
            match (v1, v2) {
                (Val::Num(n1), Val::Num(n2)) => {
                    if n1 <= n2 {
                        return (IRExpr::Val(Val::True), false);
                    } else {
                        return (IRExpr::Val(Val::False), false);
                    }
                }
                _ => (e.clone(), true),
            }
        }
        IRExpr::IsNum(v) => {
            match v {
                Val::Num(_) => (IRExpr::Val(Val::True), false),
                Val::Var(_)|
                Val::Input   => (e.clone(), true),
                _ => (IRExpr::Val(Val::False), false),
            }
        }
        IRExpr::IsBool(v) => {
            match v {
                Val::True |
                Val::False => (IRExpr::Val(Val::True), false),
                Val::Input |
                Val::Var(_) => (e.clone(), true),
                _ => (IRExpr::Val(Val::False), false),
            }
        }
        _ => (e.clone(), true)
    }
}