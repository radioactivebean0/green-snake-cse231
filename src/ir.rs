use crate::{
    anf::*
};
use crate::syntax::{Symbol};

#[derive(Debug, Copy, Clone)]
pub enum Val {
    Num(i64),
    True,
    False,
    Var(Symbol),
    Input,
    Nil,
}

#[derive(Debug, Clone)]
pub enum IRExpr {
    Add1(Val),
    Sub1(Val),
    Plus(Val, Val),
    Minus(Val, Val),
    Times(Val, Val),
    Divide(Val, Val),
    Eq(Val, Val),
    Gt(Val, Val),
    Ge(Val, Val),
    Lt(Val, Val),
    Le(Val, Val),

    IsNum(Val),
    IsBool(Val),
    IsVec(Val),

    Print(Val),

    Call(Symbol, Vec<Val>),

    MakeVec(Val, Val),
    Vec(Vec<Val>),
    VecSet(Val, Val, Val),
    VecGet(Val, Val),
    VecLen(Val),

    Val(Val),
    PrintStack,
    Gc,
}

#[derive(Debug, Clone)]
pub enum CheckType {
    CheckIsNum(Val),
    CheckIsVec(Val),
    CheckIsNotNil(Val),
    CheckEq(Val, Val),
    CheckBounds(Val, Val),
    CheckOverflow,

}

#[derive(Debug, Clone)]
pub enum Step {
    Label(Symbol),
    If(Val, Symbol, Symbol),
    Goto(Symbol),
    Do(IRExpr),
    Set(Symbol, IRExpr),
    Check(CheckType),
}

pub struct Block {
    pub steps: Vec<Step>,
}

pub struct Def {
    pub name: Symbol,
    pub args: Vec<Symbol>,
    pub body: Block,
}

pub struct Prog {
    pub defs: Vec<Def>,
    pub main: Block,
}

// fn get_uniq_name(s: Symbol, idx: u32) -> Symbol{
//     Symbol::new (format!("uniq_{s}_{idx}"))
// }

fn new_label(l: &mut i32, s: &str) -> Symbol {
    let current = *l;
    *l += 1;
    Symbol::new(format!("{s}_{current}"))
}

pub fn anf_to_ir(p: &FlatProgram) -> Prog {
    let mut defs = Vec::new();
    for def in &p.defs {
        defs.push(anf_to_ir_def(def));
    }
    let mut i = 0;
    Prog {
        defs: defs,
        main: Block {
            steps: anf_to_ir_block(&p.main, &Symbol::new("rax"), &Symbol::new(""), &mut i),
        },
    }
}

fn anf_to_ir_def(d: &FlatDefinition) -> Def {
    let args = d.args.clone();//vec![];
    //let mut bound_vars:HashMap<Symbol, u32> = HashMap::new();
    // for arg in d.args.clone().into_iter() {
    //     bound_vars = bound_vars.update(arg, 0);
    //     args.push(get_uniq_name(arg, 0));
    // }
    let mut i = 0;
    return Def{
        name: d.name.clone(), 
        args: args, 
        body: Block {
            steps: anf_to_ir_block(&d.body, &Symbol::new("rax"), &Symbol::new(""), &mut i)
        }
    };
}

pub fn anf_to_ir_block(b: &FlatBlock, target: &Symbol, brake: &Symbol, i: &mut i32) -> Vec<Step> {
    match b {
        FlatBlock::Let(name, op, body) => {
            // let new_bound_vars;
            // let uniq_name;
            // if bound_vars.contains_key(name) {
            //     let cnt = bound_vars.get(name).unwrap();
            //     new_bound_vars = bound_vars.update(*name, cnt+1);
            //     uniq_name = get_uniq_name(*name, cnt+1);
            // } else {
            //     new_bound_vars = bound_vars.update(*name, 0);
            //     uniq_name = get_uniq_name(*name, 0);
            // }
            let mut steps = anf_to_ir_expr(op, name, brake, i);//, &new_bound_vars);
            let mut body = anf_to_ir_block(body, target, brake, i);//, &new_bound_vars);
            steps.append(&mut body);
            steps
        }
        FlatBlock::Block(bs) => {
            let mut steps = Vec::new();
            let mut index = 0;
            for b in bs {
                let ttarget;
                if index != bs.len() - 1 {
                    ttarget = Symbol::new("");
                } else {
                    ttarget = *target;
                }
                index += 1;
                let mut innersteps = anf_to_ir_block(b, &ttarget, brake, i);//, bound_vars);
                steps.append(&mut innersteps);
            }
            steps
        }
        FlatBlock::Op(op) => anf_to_ir_expr(op, target, brake, i),//, bound_vars),
    }
}

pub fn anf_to_ir_expr(op: &FlatOp, target: &Symbol, brake: &Symbol, i: &mut i32) -> Vec<Step> {
    match op {
        FlatOp::If(v, b1, b2) => {
            /*
               This is the most interesting case of the ANF to IR translation.

               The key case we are considering is

               (let (x (if v b1 b2)) body)

               We want this resulting structure:

               if v thn els
               thn:
                   b2
                   break end
               els:
                   b1
               end:
               x = ***the answer of either b1 or b2 somehow***
               body

               What we end up doing is relying on x being the target of *both*
               subexpressions of the if, so if either evaluates it will assign
               into that variable at the end.

            */
            let v = anf_to_ir_val(v);//, bound_vars);
            let mut b1 = anf_to_ir_block(b1, target, brake, i);//, bound_vars);
            let mut b2 = anf_to_ir_block(b2, target, brake, i);//, bound_vars);
            let end = new_label(i, "ifend");
            let thn = new_label(i, "thn");
            let els = new_label(i, "els");

            let mut steps = Vec::new();
            steps.push(Step::If(v, thn.clone(), els.clone()));
            steps.push(Step::Label(thn.clone()));
            steps.append(&mut b1);
            steps.push(Step::Goto(end.clone()));
            steps.push(Step::Label(els.clone()));
            steps.append(&mut b2);
            steps.push(Step::Goto(end.clone()));
            steps.push(Step::Label(end.clone()));
            steps
        }
        FlatOp::Break(v) => {
            if brake.to_string() == "" {
                raise_break_outside_loop()
            }
            let v = anf_to_ir_val(v);//, bound_vars);
            vec![
                target_step(target, IRExpr::Val(v)),
                Step::Goto(*brake),
            ]
        }
        FlatOp::Loop(e) => {
            let loop_label = new_label(i, "loop");
            let end_label = new_label(i, "end");
            let mut steps = anf_to_ir_block(e, target, &end_label, i);//, bound_vars);
            steps.insert(0, Step::Label(loop_label.clone()));
            steps.push(Step::Goto(loop_label.clone()));
            steps.push(Step::Label(end_label.clone()));
            steps
        }
        FlatOp::Add1(v) => {
            let v = anf_to_ir_val(v);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v)),
                 target_step(target, IRExpr::Add1(v)),
                 Step::Check(CheckType::CheckOverflow)]
        }
        FlatOp::Sub1(v) => {
            let v = anf_to_ir_val(v);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v)),
                 target_step(target, IRExpr::Sub1(v)),
                 Step::Check(CheckType::CheckOverflow)]
        }
        FlatOp::Plus(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 Step::Check(CheckType::CheckIsNum(v2)),
                 target_step(target, IRExpr::Plus(v1, v2)),
                 Step::Check(CheckType::CheckOverflow)]
        }
        FlatOp::Minus(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 Step::Check(CheckType::CheckIsNum(v2)),
                 target_step(target, IRExpr::Minus(v1, v2)),
                 Step::Check(CheckType::CheckOverflow)]
        }
        FlatOp::Times(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 Step::Check(CheckType::CheckIsNum(v2)),
                 target_step(target, IRExpr::Times(v1, v2)),
                 Step::Check(CheckType::CheckOverflow)]
        }
        FlatOp::Divide(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 Step::Check(CheckType::CheckIsNum(v2)),
                 target_step(target, IRExpr::Divide(v1, v2)),
                 Step::Check(CheckType::CheckOverflow)]
        }
        FlatOp::Eq(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckEq(v1, v2)),
                 target_step(target, IRExpr::Eq(v1, v2))]
        }
        FlatOp::Gt(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 Step::Check(CheckType::CheckIsNum(v2)),
                 target_step(target, IRExpr::Gt(v1, v2))]
        }
        FlatOp::Ge(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 Step::Check(CheckType::CheckIsNum(v2)),
                 target_step(target, IRExpr::Ge(v1, v2))]
        }
        FlatOp::Lt(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 Step::Check(CheckType::CheckIsNum(v2)),
                 target_step(target, IRExpr::Lt(v1, v2))]
        }
        FlatOp::Le(v1, v2) => {
            let v1 = anf_to_ir_val(v1);//bound_vars);
            let v2 = anf_to_ir_val(v2);//bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 Step::Check(CheckType::CheckIsNum(v2)),
                 target_step(target, IRExpr::Le(v1, v2))]
        }
        FlatOp::IsNum(v) => {
            let v = anf_to_ir_val(v);//bound_vars);
            vec![target_step(target, IRExpr::IsNum(v))]
        }
        FlatOp::IsBool(v) => {
            let v = anf_to_ir_val(v);//bound_vars);
            vec![target_step(target, IRExpr::IsBool(v))]
        }
        FlatOp::IsVec(v) => {
            let v = anf_to_ir_val(v);//bound_vars);
            vec![target_step(target, IRExpr::IsVec(v))]
        }
        FlatOp::Print(v) => {
            let v = anf_to_ir_val(v);//bound_vars);
            vec![target_step(target, IRExpr::Print(v))]
        }
        FlatOp::Set(name, v) => {
            let v = anf_to_ir_val(v);//bound_vars);
            vec![
                Step::Set(name.clone(), IRExpr::Val(v)),
                target_step(target, IRExpr::Val(Val::Var(name.clone()))),
            ]
        }
        FlatOp::Call(name, args) => {
            let mut argvals = vec![];
            for a in args {
                argvals.push(anf_to_ir_val(a));//, bound_vars));
            }
            vec![target_step(target, IRExpr::Call(name.clone(), argvals))]
        }
        FlatOp::MakeVec(len, val) => {
            let v1 = anf_to_ir_val(len);//, bound_vars);
            let v2 = anf_to_ir_val(val);//, bound_vars);
            vec![Step::Check(CheckType::CheckIsNum(v1)),
                 target_step(target, IRExpr::MakeVec(v1, v2))]
        }
        FlatOp::Vec(vals) => {
            let mut vecvals = vec![];
            for v in vals {
                vecvals.push(anf_to_ir_val(v));//bound_vars));
            }
            vec![target_step(target, IRExpr::Vec(vecvals))]
        }
        FlatOp::VecSet(vec, idx, val) => {
            let v1 = anf_to_ir_val(vec);//, bound_vars);
            let v2 = anf_to_ir_val(idx);//, bound_vars);
            let v3 = anf_to_ir_val(val);//, bound_vars);
            vec![
                 Step::Check(CheckType::CheckBounds(v1, v2)),
                 target_step(target, IRExpr::VecSet(v1, v2, v3))]
        }
        FlatOp::VecGet(vec, idx) => {
            let v1 = anf_to_ir_val(vec);//, bound_vars);
            let v2 = anf_to_ir_val(idx);//, bound_vars);
            vec![
                 Step::Check(CheckType::CheckBounds(v1, v2)),
                 target_step(target, IRExpr::VecGet(v1, v2))]
        }
        FlatOp::VecLen(vec) => {
            let v = anf_to_ir_val(vec);//, bound_vars);
            vec![Step::Check(CheckType::CheckIsVec(v)),
                 Step::Check(CheckType::CheckIsNotNil(v)),
                 target_step(target, IRExpr::VecLen(v))]
        }
        FlatOp::Val(v) => vec![target_step(target, IRExpr::Val(anf_to_ir_val(v)))],//bound_vars))))],
        FlatOp::Input => vec![target_step(target, IRExpr::Val(Val::Input))],
        FlatOp::Nil => vec![target_step(target, IRExpr::Val(Val::Nil))],
        FlatOp::PrintStack => vec![Step::Do(IRExpr::PrintStack)],
        FlatOp::Gc => vec![Step::Set(Symbol::new("r15"), IRExpr::Gc)],
    }
}

pub fn anf_to_ir_val(v: &FlatVal) -> Val {
    match v {
        FlatVal::Num(n) => Val::Num(*n),
        FlatVal::True => Val::True,
        FlatVal::False => Val::False,
        FlatVal::Var(id) => Val::Var(id.clone()),
        //     match bound_vars.get(id) {
        //         Some(idx) => Val::Var(get_uniq_name(*id, *idx)),
        //         None => Val::Var(id.clone()),
        //     }
        // }
    }
}

fn target_step(target: &Symbol, e: IRExpr) -> Step {
    if target.to_string() == "" {
        Step::Do(e)
    } else {
        Step::Set(target.clone(), e)
    }
}

fn raise_break_outside_loop() {
    panic!("break outside loop")
}

pub fn ir_to_string(p : &Prog) -> String {
    let mut s = String::new();
    for def in &p.defs {
        s.push_str(&def_to_string(def));
    }
    s.push_str(&block_to_string(&p.main));
    s 
}

fn def_to_string(d: &Def) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}(", d.name));
    let mut idx = 0;
    for arg in d.args.clone().into_iter() {
        if idx == d.args.len()-1 {
            s.push_str(&format!("{}) {{\n", arg));
        }else {
            s.push_str(&format!("{},", arg));
            idx += 1;
        }
    }
    s.push_str(&block_to_string(&d.body));
    s.push_str("}\n\n");
    return s;
}

fn block_to_string(b : &Block) -> String {
    let mut s = String::new();
    for step in &b.steps {
        match step {
            Step::Label(l) => {
                s.push_str(&format!("\n{}:\n", l));
            }
            Step::If(v, l, r) => {
                s.push_str(&format!("if\t{} {} {}\n", val_to_string(v), l, r));
            }
            Step::Goto(l) => {
                s.push_str(&format!("goto\t{}\n", l));
            }
            Step::Do(e) => {
                s.push_str(&format!("{}\n", expr_to_string(e)));
            }
            Step::Set(name, e) => {
                s.push_str(&format!("{}\t<- {}\n", name, expr_to_string(e)));
            }
            Step::Check(ctype) => {
                match ctype {
                    CheckType::CheckIsNum(v) => s.push_str(&format!("CHECKISNUM {}\n", val_to_string(v))),
                    CheckType::CheckIsVec(v) => s.push_str(&format!("CHECKISVEC {}\n", val_to_string(v))),
                    CheckType::CheckIsNotNil(v) => s.push_str(&format!("CHECKISNOTNIL {}\n", val_to_string(v))),
                    CheckType::CheckEq(v1, v2) => s.push_str(&format!("CHECKEQ {} {}\n", val_to_string(v1), val_to_string(v2))),
                    CheckType::CheckBounds(v1, v2) => s.push_str(&format!("CHECKBOUNDS {} {}\n", val_to_string(v1), val_to_string(v2))),
                    CheckType::CheckOverflow => s.push_str(&format!("CHECKOVERFLOW\n")),
                }
            },
        }
    }
    s
}

fn expr_to_string(e : &IRExpr) -> String {
    match e {
        IRExpr::Add1(v) => format!("add1 {}", val_to_string(v)),
        IRExpr::Sub1(v) => format!("sub1 {}", val_to_string(v)),
        IRExpr::Plus(v1, v2) => format!("{} + {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Minus(v1, v2) => format!("{} - {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Eq(v1, v2) => format!("{} == {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Lt(v1, v2) => format!("{} < {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Print(v) => format!("print {}", val_to_string(v)),
        IRExpr::Val(v) => val_to_string(v),
        IRExpr::Times(v1, v2) => format!("{} * {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Divide(v1, v2) => format!("{} / {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Gt(v1, v2) => format!("{} > {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Ge(v1, v2) => format!("{} >= {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Le(v1, v2) => format!("{} <= {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::IsNum(v) => format!("isNum {}", val_to_string(v)),
        IRExpr::IsBool(v) => format!("isBool {}", val_to_string(v)),
        IRExpr::IsVec(v) => format!("isVec {}", val_to_string(v)),
        IRExpr::Call(n, args) => {
            let mut s = String::new();
            s.push_str(&format!("{}(", n));
            let mut idx = 0;
            for a in args {
                if idx == args.len()-1 {
                    s.push_str(&format!("{})", val_to_string(a)));
                } else {
                    s.push_str(&format!("{},", val_to_string(a)));
                    idx += 1;
                }
            }
            s
        },
        IRExpr::MakeVec(v1, v2) => format!("make-vec {} {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::Vec(vs) => {
            let mut s = String::new();
            s.push_str(&format!("vec("));
            let mut idx = 0;
            for val in vs {
                if idx == vs.len()-1 {
                    s.push_str(&format!("{})", val_to_string(val)));
                } else {
                    s.push_str(&format!("{},", val_to_string(val)));
                    idx += 1;
                }
            }
            s
        },
        IRExpr::VecSet(v1, v2, v3) => format!("vec-set {} {} {}", val_to_string(v1), val_to_string(v2), val_to_string(v3)),
        IRExpr::VecGet(v1, v2) =>  format!("vec-get {} {}", val_to_string(v1), val_to_string(v2)),
        IRExpr::VecLen(v) => format!("vec-len {}", val_to_string(v)),
        IRExpr::PrintStack => format!("PRINTSTACK"),
        IRExpr::Gc => format!("GC"),
    }
}

fn val_to_string(v : &Val) -> String {
    match v {
        Val::Num(n) => format!("{}", n),
        Val::True => format!("true"),
        Val::False => format!("false"),
        Val::Var(s) => format!("{}", s),
        Val::Input => format!("input"),
        Val::Nil => format!("nil"),
    }
}