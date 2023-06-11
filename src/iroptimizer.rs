use std::collections::VecDeque;
use std::env::var;

use im::HashMap;
use std::collections::HashMap as MutMap;

use crate::syntax::{Symbol};
use crate::ir::*;

pub fn optimize_ir(prog: &Prog) -> Prog {
    let (mut new_prog, mut done) = fold_constants(prog);
    while !done {
        (new_prog, done) = fold_constants(&new_prog);
        //print!("{}", ir_to_string(&new_prog));
    }
    done = false;
    while !done {
        new_prog = dead_code_elim(&new_prog);
        (new_prog, done) = fold_constants(&new_prog);
    }
    new_prog = propogate_constants(&new_prog);
    print!("{}", ir_to_string(&new_prog));
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

fn dead_code_elim(prog: &Prog) -> Prog {
    let label_map = generate_label_map(prog);
    let mut to_visit = VecDeque::new(); // queue of pairs, (func, IR step) --- main is 0, defs start 1,2,...
    let mut visited:Vec<Vec<bool>> = Vec::new();
    visited.push(Vec::new());
    visited.last_mut().unwrap().resize(prog.main.steps.len(), false);
    for def in prog.defs.as_slice() {
        visited.push(Vec::new());
        visited.last_mut().unwrap().resize(def.body.steps.len(), false);
    }
    to_visit.push_back((0,0));
    while !to_visit.is_empty() {
        let (nxt_fn, nxt_idx) = to_visit.pop_front().unwrap();
        let steps;
        if nxt_fn == 0 {
            steps = prog.main.steps.clone();
        } else {
            steps = prog.defs.get(nxt_fn-1).unwrap().body.steps.clone();
        }
        for st_num in nxt_idx..steps.len() { // visit next step
            let t_visited = visited.get_mut(nxt_fn).unwrap();
            let _ = std::mem::replace(&mut t_visited[st_num], true);
            match &steps[st_num]{
                Step::If(v, l1, l2) => {
                    match v {
                        Val::Var(_) |
                        Val::Input => {

                            match label_map.get(&l1) {
                                Some((j,k)) => {
                                    if !visited[*j][*k]{
                                        to_visit.push_back((*j,*k));
                                    }
                                }
                                None => panic!("unknown label"),
                            }
                            match label_map.get(&l2) {
                                Some((j,k)) => {
                                    if !visited[*j][*k]{
                                        to_visit.push_back((*j,*k));
                                    }
                                }
                                None => panic!("unknown label"),
                            }
                            if nxt_fn == 0{
                                println!("visiting both branches");
                                println!("{},{}", l1, l2);
                                println!("{:?}",to_visit);
                            }
                            break;
                        },
                        Val::False => {
                            let _ = std::mem::replace(&mut t_visited[st_num], false);
                            match label_map.get(&l2) {
                                Some((j,k)) => {
                                    if !visited[*j][*k]{
                                        to_visit.push_back((*j,*k));
                                    }
                                }
                                None => panic!("unknown label"),
                            }
                            break;
                        },
                        _ => {
                            let _ = std::mem::replace(&mut t_visited[st_num], false);
                            match label_map.get(&l1) {
                                Some((j,k)) => {
                                    if !visited[*j][*k]{
                                        to_visit.push_back((*j,*k));
                                    }
                                }
                                None => panic!("unknown label"),
                            }
                            break;
                        },

                    }
                }
                Step::Goto(lbl) => {
                    match label_map.get(&lbl) {
                        Some((j,k)) => {
                            if !visited[*j][*k]{
                                to_visit.push_back((*j,*k));
                            }
                        }
                        None => panic!("unknown label"),
                    }
                    break;
                }
                Step::Do(e) => {
                    match e {
                        IRExpr::Call(n,_) => {
                            match label_map.get(n) {
                                Some((j,k)) => {
                                    if !visited[*j][*k]{
                                        to_visit.push_back((*j,*k));
                                    }
                                }
                                None => panic!("unknown label"),
                            }
                        },
                        _ => (),
                    }
                }
                Step::Set(_, e) => {
                    match e {
                        IRExpr::Call(n,_) => {
                            match label_map.get(&n) {
                                Some((j,k)) => {
                                    if !visited[*j][*k] {
                                        to_visit.push_back((*j,*k));
                                    }
                                }
                                None => panic!("unknown label"),
                            }
                        },
                        _ => (),
                    }
                },
                _ => ()
            }
        }
    }

    let mut new_main = vec![];
    let mut idx = 0;
    let main_visited = &visited[0];
    for step in prog.main.steps.clone() {
        if main_visited[idx] {
            match step {
                Step::Goto(x) => {
                    match label_map.get(&x) {
                        Some((j,k)) => {
                            if visited[*j][*k] {
                                new_main.push(step.clone())
                            }
                        }
                        None => todo!(),
                    }
                },
                _ =>  new_main.push(step.clone()),
            }
        }
        idx += 1;
    }
    let mut new_defs = vec![];
    let didx = 1;
    idx = 0;
    for def in prog.defs.as_slice() {
        let mut new_def_steps = vec![];
        let def_visited = &visited[didx];
        for step in def.body.steps.as_slice() {
            if def_visited[idx] {
                match step {
                    Step::Goto(x) => {
                        match label_map.get(&x) {
                            Some((j,k)) => {
                                if visited[*j][*k] {
                                    new_def_steps.push(step.clone())
                                }
                            }
                            None => todo!(),
                        }
                    },
                    _ =>  new_def_steps.push(step.clone()),
                }
            }
        }
        new_defs.push(Def{name: def.name.clone(), args: def.args.clone(), body: Block{steps: new_def_steps}});
    }

    return Prog{defs: new_defs, main: Block{steps: new_main}};
}

fn generate_label_map(prog: &Prog) -> HashMap<Symbol, (usize,usize)> {
    let mut points = find_labels_block(&prog.main, &HashMap::new(), 0);
    points = find_labels_defs(&points, &prog.defs);
    return points;
}

fn find_labels_defs(points: &HashMap<Symbol,(usize,usize)>, defs: &[Def]) -> HashMap<Symbol, (usize, usize)>{
    let mut dnum = 1;
    let mut acc = points.clone();
    for def in defs {
        acc = find_labels_block(&def.body, &acc, dnum).update(def.name.clone(), (dnum, 0));
        dnum += 1;
    }
    return acc;
}

fn find_labels_block(block: &Block, curr_pts: &HashMap<Symbol, (usize, usize)>, bnum: usize) -> HashMap<Symbol, (usize,usize)> {
    let mut b_labels = curr_pts.clone();
    for i in 0..block.steps.len() {
        let s = &block.steps[i];
        match s {
            Step::Label(l) => {
                b_labels = b_labels.update(l.clone(), (bnum, i));
            },
            _ => (),
        }
    }
    return b_labels;
}
fn is_hard_coded_reg (s: &Symbol) -> bool{
    match s.to_string().as_str() {
        "rax"|
        "r15" => true,
        _ => false,
    }
}
fn propogate_constants(prog: &Prog) -> Prog {
    return Prog {
        defs: propogate_constants_defs(&prog.defs),
        main: propogate_constants_block(&prog.main, &vec![])
    }
}

fn propogate_constants_defs(defs: &[Def]) -> Vec<Def> {
    let mut new_defs = vec![];
    for def in defs {
        new_defs.push(Def{name: def.name.clone(), args: def.args.clone(), body: propogate_constants_block(&def.body, &def.args)});
    }
    return new_defs;
}

fn propogate_constants_block(block: &Block, args: &[Symbol]) -> Block{
    let mut var_map:MutMap<Symbol, Val> = MutMap::new();
    let mut to_rm = vec![];
    for step in block.steps.as_slice() {
        match step {
            Step::Set(x, v) => {
                if !is_hard_coded_reg(x){
                    match v {
                        IRExpr::Val(val) => {
                            match var_map.get(&x){
                                Some(v) => if ! (*v == *val) {
                                    to_rm.push(x.clone());
                                    // let t = var_map.remove_entry(&x);
                                    // println!("removed {:?}",t.unwrap());
                                    // println!("{:?}",var_map);
                                },
                                None => _ = var_map.insert(x.clone(), *val),
                            }
                        },
                        _ => {
                            match var_map.get(&x){
                                Some(_) => {
                                    to_rm.push(x.clone());
                                    //_ = var_map.remove_entry(&x);
                                },
                                None => to_rm.push(x.clone()),

                            }
                        },
                    }
                }
            }
            _ => (),
        }
    }
    println!("{:?}", to_rm);
    for s in to_rm {
        _ = var_map.remove(&s);
    }
    for a in args {
        _ = var_map.remove(&a);
    }
    println!("varmap: {:?}", var_map);
    let mut new_steps = vec![];
    for step in block.steps.as_slice() {
        match step {
            Step::Label(_)|
            Step::Goto(_) => new_steps.push(step.clone()),
            Step::If(v, l1, l2) => {
                match v {
                    Val::Var(x) => {
                        match var_map.get(&x){
                            Some(v) => new_steps.push(Step::If(v.clone(),l1.clone(),l2.clone())),
                            None => new_steps.push(step.clone()),
                        }

                    }
                    _ => new_steps.push(step.clone()),
                }
            },
            Step::Do(e) => new_steps.push(Step::Do(propogate_constants_expr(&e, &var_map))),
            Step::Set(x, e) => {
                if var_map.contains_key(&x) {
                    continue;
                } else {
                    new_steps.push(Step::Set(x.clone(), propogate_constants_expr(&e, &var_map)));
                }
            }
            Step::Check(t) => {
                match t {
                    CheckType::CheckIsNum(v) => new_steps.push(Step::Check(CheckType::CheckIsNum(propogate_constants_val(&v, &var_map)))),
                    CheckType::CheckIsVec(v) => new_steps.push(Step::Check(CheckType::CheckIsVec(propogate_constants_val(&v, &var_map)))),
                    CheckType::CheckIsNotNil(v) => new_steps.push(Step::Check(CheckType::CheckIsNotNil(propogate_constants_val(&v, &var_map)))),
                    CheckType::CheckEq(v1,v2) => new_steps.push(Step::Check(CheckType::CheckEq(propogate_constants_val(&v1, &var_map), propogate_constants_val(&v2, &var_map)))),
                    CheckType::CheckBounds(v1,v2) => new_steps.push(Step::Check(CheckType::CheckBounds(propogate_constants_val(&v1, &var_map), propogate_constants_val(&v2, &var_map)))),
                    CheckType::CheckOverflow => new_steps.push(Step::Check(CheckType::CheckOverflow)),
                }
            },
        }
    }
    return Block{steps: new_steps};
}

fn propogate_constants_expr(expr: &IRExpr, var_map: &MutMap<Symbol, Val>) -> IRExpr {
    match expr{
        IRExpr::Add1(v) => IRExpr::Add1(propogate_constants_val(v, var_map)),
        IRExpr::Sub1(v) => IRExpr::Sub1(propogate_constants_val(v, var_map)),
        IRExpr::Plus(v1, v2) => IRExpr::Plus(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Minus(v1, v2) => IRExpr::Minus(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Times(v1, v2) => IRExpr::Times(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Divide(v1, v2) => IRExpr::Divide(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Eq(v1, v2) => IRExpr::Eq(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Gt(v1, v2) => IRExpr::Gt(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Ge(v1, v2) => IRExpr::Ge(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Lt(v1, v2) => IRExpr::Lt(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Le(v1, v2) => IRExpr::Le(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::IsNum(v) => IRExpr::IsNum(propogate_constants_val(v, var_map)),
        IRExpr::IsBool(v) => IRExpr::IsBool(propogate_constants_val(v, var_map)),
        IRExpr::IsVec(v) => IRExpr::IsVec(propogate_constants_val(v, var_map)),
        IRExpr::Print(v) => IRExpr::Print(propogate_constants_val(v, var_map)),
        IRExpr::Call(fun, args) => {
            let mut new_args = vec![];
            for arg in args {
                new_args.push(propogate_constants_val(arg, var_map));
            }
            IRExpr::Call(fun.clone(), new_args)
        }
        IRExpr::MakeVec(v1, v2) => IRExpr::MakeVec(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::Vec(vs) => {
            let mut new_v = vec![];
            for v in vs {
                new_v.push(propogate_constants_val(v, var_map));
            }
            println!("{:?}",new_v);
            IRExpr::Vec(new_v)
        }
        IRExpr::VecSet(v1, v2, v3) => IRExpr::VecSet(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map), propogate_constants_val(v3, var_map)),
        IRExpr::VecGet(v1, v2) => IRExpr::VecGet(propogate_constants_val(v1, var_map), propogate_constants_val(v2, var_map)),
        IRExpr::VecLen(v) => IRExpr::VecLen(propogate_constants_val(v, var_map)),
        IRExpr::Val(v) => IRExpr::Val(propogate_constants_val(v, var_map)),
        _ => expr.clone(),
    }
}
fn propogate_constants_val(val: &Val, var_map: &MutMap<Symbol, Val>) -> Val {
    match val {
        Val::Var(x) => { 
            match var_map.get(&x){
                Some(v) => v.clone(),
                None => val.clone(),
            }
        },
        _ => val.clone(),
    }
}