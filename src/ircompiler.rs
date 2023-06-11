use std::collections::{HashMap as MutableMap};

use crate::ir::*;
use crate::syntax::{Symbol};
use crate::{
    asm::{
        instrs_to_string, Arg32, Arg64, BinArgs, CMov, Instr, Loc, MemRef, MovArgs, Offset,
        Reg::{self, *},
        Reg32,
        StrOp::Stosq,
    }};
use crate::mref;

const INVALID_ARG: &str = "invalid_argument";
const OVERFLOW: &str = "overflow";
const INDEX_OUT_OF_BOUNDS: &str = "index_out_of_bounds";
const INVALID_SIZE: &str = "invalid_vec_size";

const STACK_BASE: Reg = Rbx;
const INPUT_REG: Reg = R13;
const HEAP_END: Reg = R14;
const HEAP_PTR: Reg = R15;
const CHECK_REG: Reg = Rdx;
const CHECK_REG2: Reg = R10;

const NIL: i32 = 0b001;
const MEM_SET_VAL: i32 = NIL;
const GC_WORD_VAL: i32 = 0;

struct IRSession {
    instrs: Vec<Instr>,
    funs: MutableMap<Symbol,usize>,
    tag: u32
}

pub fn compile_ir_prog(prg: &Prog) -> String {
    let mut funs:MutableMap<Symbol, usize> = MutableMap::new();
    for def in &prg.defs[..] {
        funs.insert(def.name, def.args.len());
    }
    let mut sess = IRSession::new(funs);
    sess.compile_defs(&prg.defs);
    sess.emit_instr(Instr::Label("our_code_starts_here".to_string()));
    let callee_saved = [Rbp, STACK_BASE, INPUT_REG, HEAP_END, HEAP_PTR];
    let mut env = sess.fun_entry(&prg.main, &vec![], &callee_saved);
    sess.emit_instrs([
        Instr::Mov(MovArgs::ToReg(STACK_BASE, Arg64::Reg(Rbp))),
        Instr::Mov(MovArgs::ToReg(INPUT_REG, Arg64::Reg(Rdi))),
        Instr::Mov(MovArgs::ToReg(HEAP_PTR, Arg64::Reg(Rsi))),
        Instr::Mov(MovArgs::ToReg(HEAP_END, Arg64::Reg(Rdx))),
    ]);
    //let env = calc_env(&prg.main);
    sess.compile_ir_block(&prg.main, &mut env, &Symbol::new("main"));
    sess.fun_exit(&env, &callee_saved);
    format!(
                "
section .text
extern snek_error
extern snek_print
extern snek_alloc_vec
extern snek_print_stack
extern snek_try_gc
extern snek_gc
global our_code_starts_here
{}
{INVALID_ARG}:
  mov edi, 1
  call snek_error
{OVERFLOW}:
  mov edi, 2
  call snek_error
{INDEX_OUT_OF_BOUNDS}:
  mov edi, 3
  call snek_error
{INVALID_SIZE}:
  mov edi, 4
  call snek_error
",                 instrs_to_string(&sess.instrs))
}

impl IRSession {
    fn new(funs: MutableMap<Symbol, usize>) -> IRSession {
        IRSession { instrs: vec![], funs: funs, tag: 0 }
    }

    fn fun_entry(&mut self, b: &Block, args: &Vec<Symbol>, callee_saved: &[Reg]) -> MutableMap<Symbol, i32>{
        let mut env = MutableMap::new();
        for reg in callee_saved {
            self.emit_instr(Instr::Push(Arg32::Reg(*reg)));
        }
        for step in &b.steps {
            match step {
                Step::Set(x, _) => {
                    if !env.contains_key(x) {
                        let offset = (env.len()) as i32;
                        env.insert(x.clone(), offset + 1);
                    }
                }
                _ => {}
            }
        }
        for i in 0..args.len() {
            env.insert(args[i].clone(), (-1-(callee_saved.len() as i32))-(i as i32));
        }
        let mut size = env.len() + callee_saved.len()+1;
        if size % 2 == 0 {
            size = env.len();
        } else {
            size = env.len()+1
        }
        self.emit_instrs([
            Instr::Mov(MovArgs::ToReg(Rbp, Arg64::Reg(Rsp))),
            Instr::Sub(BinArgs::ToReg(Rsp, Arg32::Imm(8 * (size as i32)))),
        ]);
        self.memset(0, size as u32, Reg32::Imm(MEM_SET_VAL));
        env
    }
    
    fn fun_exit(&mut self, env: &MutableMap<Symbol, i32>, callee_saved: &[Reg]) {
        let mut size = env.len() + callee_saved.len()+1;
        if size % 2 == 0 {
            size = env.len();
        } else {
            size = env.len()+1
        }
        self.emit_instrs([Instr::Add(BinArgs::ToReg(
            Rsp,
            Arg32::Imm(8 * (size as i32)),
        ))]);
        for reg in callee_saved.iter().rev() {
            self.emit_instr(Instr::Pop(Loc::Reg(*reg)));
        }
        self.emit_instr(Instr::Ret);
    }

    fn compile_defs(&mut self, defs: &[Def]) {
        for def in defs {
            self.compile_ir_def(def, &[Rbp]);
        }
    }

    fn compile_ir_def(&mut self, d: &Def, callee_saved: &[Reg]) {
        self.emit_instr(Instr::Label(d.name.to_string()));
        let mut env = self.fun_entry(&d.body, &d.args, callee_saved);
        self.compile_ir_block(&d.body, &mut env, &d.name);
        self.fun_exit(&env, callee_saved);
    }

    fn compile_ir_block(&mut self, b : &Block, env: &mut MutableMap<Symbol, i32>, lbl: &Symbol) {
        for step in &b.steps {
            self.compile_ir_step(&step, env, lbl);
        }
    }

    fn compile_ir_step(&mut self, s : &Step, env: &mut MutableMap<Symbol, i32>, lbl : &Symbol){
        match s {
            Step::Label(l) => self.emit_instr(Instr::Label(format!("{lbl}_{l}"))),
            Step::If(v, thn, els) => {
                self.compile_ir_val(&v, Loc::Reg(Rax), env);
                self.emit_instrs([
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Imm(3))),
                    Instr::Je(format!("{lbl}_{els}")),
                    Instr::Jmp(format!("{lbl}_{thn}"))
                ]);
            }
            Step::Goto(l) => self.emit_instr(Instr::Jmp(format!("{lbl}_{l}"))),
            Step::Do(e) => self.compile_ir_expr(e, env),
            Step::Set(x, e) => {
                let offset = match env.get(x) {
                    Some(offset) => (*offset) * 8,
                    None => {
                        panic!("Unbound identifier {x}")
                    }
                };
                self.compile_ir_expr(e, env);
                self.emit_instr(Instr::Mov(MovArgs::ToMem(mref![Rbp - %(offset)], Reg32::Reg(Rax))));
            }
            Step::Check(ctype) => {
                match ctype {
                    CheckType::CheckIsNum(v) => {
                        match v {
                            Val::Num(_) => return,
                            Val::Input => {
                                self.emit_instrs([
                                    Instr::Test(BinArgs::ToReg(Rdi, Arg32::Imm(0b001))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                ]);
                            },
                            Val::Var(var) => {
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b001))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                ]);
                            },
                            _ => self.emit_instr(Instr::Jmp(INVALID_ARG.to_string())),
                        }
                    },
                    CheckType::CheckIsVec(v) => {
                        match v {
                            Val::Var(var) => {
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b001))),
                                    Instr::Jz(INVALID_ARG.to_string()), // jump if is num
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b010))),
                                    Instr::Jnz(INVALID_ARG.to_string()), // jump if is bool
                                ]);
                            },
                            Val::Nil => return,
                            _ => self.emit_instr(Instr::Jmp(INVALID_ARG.to_string())),
                        }
                    },
                    CheckType::CheckIsNotNil(v) => {
                        match v {
                            Val::Nil => return,
                            Val::Var(var) => {
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Imm(NIL))),
                                    Instr::Jz(INVALID_ARG.to_string()), // jump if exactly equal to 1
                                ]);
                            }
                            _ => self.emit_instr(Instr::Jmp(INVALID_ARG.to_string())),
                        }
                    },
                    CheckType::CheckEq(v1, v2) => {
                        match (v1, v2) {
                            (Val::False, Val::True) |
                            (Val::True, Val::False) |
                            (Val::True, Val::True) |
                            (Val::False, Val::False) |
                            (Val::Input, Val::Input) |
                            (Val::Num(_), Val::Num(_)) |
                            (Val::Nil, Val::Nil) => {
                                return
                            }
                            (Val::Var(var), Val::Num(_))|
                            (Val::Num(_), Val::Var(var)) => {
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b001))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                ]);
                            }
                            (Val::Var(var), Val::False) |
                            (Val::Var(var), Val::True) |
                            (Val::False, Val::Var(var)) |
                            (Val::True, Val::Var(var)) => {
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::And(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b011))),
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b011))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                ]);
                            }
                            (Val::Var(var), Val::Nil) |
                            (Val::Nil, Val::Var(var)) => {
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b001))),
                                    Instr::Jz(INVALID_ARG.to_string()), // jump if is num
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b010))),
                                    Instr::Jnz(INVALID_ARG.to_string()), // jump if is bool
                                ]);
                            }
                            (Val::Input, Val::Var(var)) |
                            (Val::Var(var), Val::Input) =>{
                                let tag = self.next_tag();
                                let check_eq_finish_lbl = format!("check_eq_finish_{tag}");
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Xor(BinArgs::ToReg(CHECK_REG, Arg32::Reg(Rdi))),
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b11))),
                                    Instr::Jz(check_eq_finish_lbl.to_string()),
                                ]);
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Or(BinArgs::ToReg(CHECK_REG, Arg32::Reg(Rdi))),
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b01))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                    Instr::Label(check_eq_finish_lbl.to_string()),
                                ]);
                            },
                            (Val::Input, Val::Num(_)) |
                            (Val::Num(_), Val::Input) => {
                                self.emit_instrs([
                                    Instr::Test(BinArgs::ToReg(Rdi, Arg32::Imm(0b001))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                ]);                            
                            },
                            (Val::True, Val::Input) |
                            (Val::False, Val::Input) |
                            (Val::Input, Val::True) |
                            (Val::Input, Val::False) => {
                                self.emit_instrs([
                                    Instr::Mov(MovArgs::ToReg(CHECK_REG, Arg64::Reg(Rdi))),
                                    Instr::And(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b011))),
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b011))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                ]);                            
                            },
                            (Val::Var(var1), Val::Var(var2)) => {
                                let tag = self.next_tag();
                                let check_eq_finish_lbl = format!("check_eq_finish_{tag}");
                                self.compile_ir_var(var1.clone(), Loc::Reg(CHECK_REG), env);
                                self.compile_ir_var(var2.clone(), Loc::Reg(CHECK_REG2), env);
                                self.emit_instrs([
                                    Instr::Xor(BinArgs::ToReg(CHECK_REG, Arg32::Reg(CHECK_REG2))),
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b11))),
                                    Instr::Jz(check_eq_finish_lbl.to_string()),
                                ]);
                                self.compile_ir_var(var1.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Or(BinArgs::ToReg(CHECK_REG, Arg32::Reg(CHECK_REG2))),
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b01))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                    Instr::Label(check_eq_finish_lbl.to_string()),
                                ]);
                            },
                            (_, _) => self.emit_instr(Instr::Jmp(INVALID_ARG.to_string())),
                        }
                    },
                    CheckType::CheckBounds(v1, v2) => {
                        match (v1, v2) {
                            (_, Val::False) |
                            (_, Val::True) |
                            (_, Val::Nil) => self.emit_instr(Instr::Jmp(INVALID_ARG.to_string())),
                            (Val::Num(_), _) |
                            (Val::True, _) |
                            (Val::False, _) |
                            (Val::Nil, _) |
                            (Val::Input, _) => self.emit_instr(Instr::Jmp(INVALID_ARG.to_string())),
                            (Val::Var(var), Val::Num(n)) => {
                                if *n < 0 {
                                    self.emit_instr(Instr::Jmp(INDEX_OUT_OF_BOUNDS.to_string()));
                                } else {
                                    self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                    self.emit_instrs([
                                        Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b001))),
                                        Instr::Jz(INVALID_ARG.to_string()), // jump if is num
                                        Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b010))),
                                        Instr::Jnz(INVALID_ARG.to_string()), // jump if is bool
                                        Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Imm(NIL))),
                                        Instr::Jz(INVALID_ARG.to_string()), // jump if exactly equal to 1
                                        Instr::Sub(BinArgs::ToReg(CHECK_REG, Arg32::Imm(1))),
                                        Instr::Mov(MovArgs::ToReg(CHECK_REG, Arg64::Mem(mref![CHECK_REG + 8]))),
                                        Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0))),
                                        Instr::Jl(INDEX_OUT_OF_BOUNDS.to_string()),
                                        Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Imm(*n as i32))),
                                        Instr::Jle(INDEX_OUT_OF_BOUNDS.to_string()),
                                    ]);
                                }
                            }
                            (Val::Var(var1), Val::Var(var2)) => {
                                self.compile_ir_var(var2.clone(), Loc::Reg(CHECK_REG2), env);
                                self.emit_instrs([ // test idx is a num
                                    Instr::Test(BinArgs::ToReg(CHECK_REG2, Arg32::Imm(0b001))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                ]);
                                self.compile_ir_var(var1.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b001))),
                                    Instr::Jz(INVALID_ARG.to_string()), // jump if is num
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b010))),
                                    Instr::Jnz(INVALID_ARG.to_string()), // jump if is bool
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Imm(NIL))),
                                    Instr::Jz(INVALID_ARG.to_string()), // jump if exactly equal to 1
                                    Instr::Sub(BinArgs::ToReg(CHECK_REG, Arg32::Imm(1))),
                                    Instr::Mov(MovArgs::ToReg(CHECK_REG, Arg64::Mem(mref![CHECK_REG + 8]))),
                                    Instr::Sar(BinArgs::ToReg(CHECK_REG2, Arg32::Imm(1))),
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG2, Arg32::Imm(0))),
                                    Instr::Jl(INDEX_OUT_OF_BOUNDS.to_string()),
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Reg(CHECK_REG2))),
                                    Instr::Jle(INDEX_OUT_OF_BOUNDS.to_string()),
                                ]);
                            }
                            (Val::Var(var), Val::Input) => {
                                self.emit_instrs([ // test input is a num
                                    Instr::Test(BinArgs::ToReg(Rdi, Arg32::Imm(0b001))),
                                    Instr::Jnz(INVALID_ARG.to_string()),
                                ]);
                                self.compile_ir_var(var.clone(), Loc::Reg(CHECK_REG), env);
                                self.emit_instrs([
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b001))),
                                    Instr::Jz(INVALID_ARG.to_string()), // jump if is num
                                    Instr::Test(BinArgs::ToReg(CHECK_REG, Arg32::Imm(0b010))),
                                    Instr::Jnz(INVALID_ARG.to_string()), // jump if is bool
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Imm(NIL))),
                                    Instr::Jz(INVALID_ARG.to_string()), // jump if exactly equal to 1
                                    Instr::Sub(BinArgs::ToReg(CHECK_REG, Arg32::Imm(1))),
                                    Instr::Mov(MovArgs::ToReg(CHECK_REG, Arg64::Mem(mref![CHECK_REG + 8]))),
                                    Instr::Sar(BinArgs::ToReg(CHECK_REG2, Arg32::Imm(1))),
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG2, Arg32::Imm(0))),
                                    Instr::Jl(INDEX_OUT_OF_BOUNDS.to_string()),
                                    Instr::Cmp(BinArgs::ToReg(CHECK_REG, Arg32::Reg(CHECK_REG2))),
                                    Instr::Jle(INDEX_OUT_OF_BOUNDS.to_string()),
                                ]);
                            }
                        }
                    },
                    CheckType::CheckOverflow => self.emit_instr(Instr::Jo(OVERFLOW.to_string())),
                }
            },
        }
    }

    fn compile_ir_expr(&mut self, e : &IRExpr, env: &mut MutableMap<Symbol, i32>){
        match e {
            IRExpr::Add1(e) => {
                self.compile_ir_val(&e, Loc::Reg(Rax), env);
                self.emit_instr(Instr::Add(BinArgs::ToReg(Rax, Arg32::Imm(2))));
            },
            IRExpr::Sub1(e) => {
                self.compile_ir_val(&e, Loc::Reg(Rax), env);
                self.emit_instr(Instr::Sub(BinArgs::ToReg(Rax, Arg32::Imm(2))));
            },
            IRExpr::Plus(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instr(Instr::Add(BinArgs::ToReg(Rax, Arg32::Reg(Rcx))));
            },
            IRExpr::Minus(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instr(Instr::Sub(BinArgs::ToReg(Rax, Arg32::Reg(Rcx))));
            },
            IRExpr::Times(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instrs([
                    Instr::Sar(BinArgs::ToReg(Rax, Arg32::Imm(1))),
                    Instr::IMul(BinArgs::ToReg(Rax, Arg32::Reg(Rcx)))]);
            },
            IRExpr::Divide(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instrs([
                    Instr::Cqo,
                    Instr::IDiv(Rcx),
                    Instr::Sal(BinArgs::ToReg(Rax, Arg32::Imm(1)))]);
            },
            IRExpr::Eq(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instrs([
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Reg(Rcx))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Imm(7))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Imm(3))),
                    Instr::CMov(CMov::E(Rax, Arg64::Reg(Rcx)))
                ]);
            },
            IRExpr::Gt(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instrs([
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Reg(Rcx))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Imm(7))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Imm(3))),
                    Instr::CMov(CMov::G(Rax, Arg64::Reg(Rcx)))
                ]);
            },
            IRExpr::Ge(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instrs([
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Reg(Rcx))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Imm(7))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Imm(3))),
                    Instr::CMov(CMov::GE(Rax, Arg64::Reg(Rcx)))
                ]);
            },
            IRExpr::Lt(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instrs([
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Reg(Rcx))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Imm(7))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Imm(3))),
                    Instr::CMov(CMov::L(Rax, Arg64::Reg(Rcx)))
                ]);
            },
            IRExpr::Le(e1, e2) => {
                self.compile_ir_val(&e1, Loc::Reg(Rax), env);
                self.compile_ir_val(&e2, Loc::Reg(Rcx), env);
                self.emit_instrs([
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Reg(Rcx))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Imm(7))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Imm(3))),
                    Instr::CMov(CMov::LE(Rax, Arg64::Reg(Rcx)))
                ]);
            },
            IRExpr::IsNum(v) => {
                self.compile_ir_val(&v, Loc::Reg(Rax), env);
                self.emit_instrs([
                    Instr::And(BinArgs::ToReg(Rax, Arg32::Imm(0b001))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Imm(3))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Imm(7))),
                    Instr::CMov(CMov::Z(Rax, Arg64::Reg(Rcx))),
                ]);
            },
            IRExpr::IsBool(v) => {
                self.compile_ir_val(&v, Loc::Reg(Rax), env);
                self.emit_instrs([
                    Instr::And(BinArgs::ToReg(Rax, Arg32::Imm(0b011))),
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Imm(0b011))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Imm(3))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Imm(7))),
                    Instr::CMov(CMov::E(Rax, Arg64::Reg(Rcx))),
                ]);
            },
            IRExpr::IsVec(v) => {
                self.compile_ir_val(&v, Loc::Reg(Rax), env);
                self.emit_instrs([
                    Instr::Mov(MovArgs::ToReg(Rdx, Arg64::Reg(Rax))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Imm(7))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Imm(3))),
                    Instr::Test(BinArgs::ToReg(Rdx, Arg32::Imm(0b01))),
                    Instr::CMov(CMov::Z(Rax, Arg64::Reg(Rcx))),
                    Instr::Test(BinArgs::ToReg(Rdx, Arg32::Imm(0b10))),
                    Instr::CMov(CMov::NZ(Rax, Arg64::Reg(Rcx))),
                ]);
            },
            IRExpr::Print(v) => {
                self.compile_ir_val(&v, Loc::Reg(Rax), env);
                self.emit_instrs([
                    Instr::Mov(MovArgs::ToReg(Rdi, Arg64::Reg(Rax))),
                    Instr::Call("snek_print".to_string())
                ]);
            },
            IRExpr::Call(fun, args) => {
                let Some(arity) = self.funs.get(fun) else {
                    return raise_undefined_fun(*fun);
                };
                if args.len() != *arity {
                    raise_wrong_number_of_args(*fun, *arity, args.len());
                }
                let mut argspace = args.len();
                if args.len() % 2 != 0 {
                    self.emit_instr(Instr::Push(Arg32::Imm(MEM_SET_VAL)));
                    argspace += 1;
                }
                for arg in args.iter().rev() {
                    self.compile_ir_val(arg, Loc::Reg(Rcx), env);
                    self.emit_instr(Instr::Push(Arg32::Reg(Rcx)));
                }

                self.emit_instrs([
                    Instr::Call(fun.to_string()),
                    Instr::Add(BinArgs::ToReg(Rsp, Arg32::Imm(8 * argspace as i32))),
                ]);
            },
            IRExpr::MakeVec(sz, elm) => {
                let tag = self.next_tag();
                let alloc_finish_lbl = format!("make_vec_alloc_finish_{tag}");
                self.compile_ir_val(elm, Loc::Reg(Rcx), env);
                self.compile_ir_val(sz, Loc::Reg(Rdi), env);
                self.emit_instrs([
                    Instr::Sar(BinArgs::ToReg(Rdi, Arg32::Imm(1))),
                    Instr::Cmp(BinArgs::ToReg(Rdi, Arg32::Imm(0))),
                    Instr::Jl(INVALID_SIZE.to_string()),
                    Instr::Lea(Rax, mref![HEAP_PTR + 8 * Rdi + 16]),
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Reg(HEAP_END))),
                    Instr::Jle(alloc_finish_lbl.clone()),
                    // Call try_gc to ensure we can allocate `size + 2` quad words
                    // (1 extra for the size of the vector + 1 extra for the GC metadata)
                    Instr::Add(BinArgs::ToReg(Rdi, Arg32::Imm(2))),
                    Instr::Mov(MovArgs::ToReg(Rsi, Arg64::Reg(HEAP_PTR))),
                    Instr::Mov(MovArgs::ToReg(Rdx, Arg64::Reg(STACK_BASE))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Reg(Rbp))),
                    Instr::Mov(MovArgs::ToReg(R8, Arg64::Reg(Rsp))),
                    Instr::Call("snek_try_gc".to_string()),
                    Instr::Mov(MovArgs::ToReg(HEAP_PTR, Arg64::Reg(Rax))),
                    Instr::Label(alloc_finish_lbl),
                ]);
                // Load size again in %rsi
                self.compile_ir_val(sz, Loc::Reg(Rsi), env);
                self.emit_instrs([
                    Instr::Sar(BinArgs::ToReg(Rsi, Arg32::Imm(1))),
                    // Write GC word in HEAP_PTR
                    Instr::Mov(MovArgs::ToMem(mref!(HEAP_PTR + 0), Reg32::Imm(GC_WORD_VAL))),
                    // Write size in HEAP_PTR + 8
                    Instr::Mov(MovArgs::ToMem(mref!(HEAP_PTR + 8), Reg32::Reg(Rsi))),
                    // Fill vector using `rep stosq` (%rdi = ptr, %rcx = count, %rax = val)
                    Instr::Lea(Rdi, mref!(HEAP_PTR + 16)),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Reg(Rsi))),
                ]);
                self.compile_ir_val(elm, Loc::Reg(Rax), env);
                self.emit_instrs([
                    Instr::Rep(Stosq),
                    // Add tag to heap ptr and store it in %rax as the result of the expression
                    Instr::Lea(Rax, mref!(HEAP_PTR + 1)),
                    // Bump heap ptr
                    Instr::Lea(HEAP_PTR, mref!(HEAP_PTR + 8 * Rsi + 16)),
                ]);
            },
            IRExpr::Vec(elems) => {
                let tag = self.next_tag();
                let vec_alloc_finish_lbl = format!("vec_alloc_finish_{tag}");

                let size: i32 = elems.len().try_into().unwrap();

                self.emit_instrs([
                    Instr::Lea(Rax, mref![HEAP_PTR + %(8 * (size + 2))]),
                    Instr::Cmp(BinArgs::ToReg(Rax, Arg32::Reg(HEAP_END))),
                    Instr::Jle(vec_alloc_finish_lbl.clone()),
                    // Call try_gc to ensure we can allocate `size + 2` quad words
                    // (1 extra for the size of the vector + 1 extra for the GC metadata)
                    Instr::Mov(MovArgs::ToReg(Rdi, Arg64::Imm(size as i64 + 2))),
                    Instr::Mov(MovArgs::ToReg(Rsi, Arg64::Reg(HEAP_PTR))),
                    Instr::Mov(MovArgs::ToReg(Rdx, Arg64::Reg(STACK_BASE))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Reg(Rbp))),
                    Instr::Mov(MovArgs::ToReg(R8, Arg64::Reg(Rsp))),
                    Instr::Call("snek_try_gc".to_string()),
                    Instr::Mov(MovArgs::ToReg(HEAP_PTR, Arg64::Reg(Rax))),
                    Instr::Label(vec_alloc_finish_lbl),
                    // Write GC word in HEAP_PTR
                    Instr::Mov(MovArgs::ToMem(mref!(HEAP_PTR + 0), Reg32::Imm(GC_WORD_VAL))),
                    // Write size in HEAP_PTR + 8
                    Instr::Mov(MovArgs::ToMem(mref!(HEAP_PTR + 8), Reg32::Imm(size))),
                ]);

                for i in 0..elems.len() as u32 {
                    self.compile_ir_val(&elems[(i as usize)], Loc::Reg(Rcx), env);
                    self.move_to(
                        Loc::Mem(mref!(HEAP_PTR + %(8 * (i + 2)))),
                        Arg64::Reg(Rcx),
                    )
                }

                self.emit_instrs([
                    // Add tag to heap ptr and store it in %rax as the result of the expression
                    Instr::Lea(Rax, mref!(HEAP_PTR + 1)),
                    // Bump heap ptr
                    Instr::Lea(HEAP_PTR, mref!(HEAP_PTR + %(8 * (size + 2)))),
                ]);
            },
            IRExpr::VecSet(vec, idx, elem) => {
                self.compile_ir_val(elem, Loc::Reg(Rcx), env);
                self.compile_ir_val(vec, Loc::Reg(Rax), env);
                self.compile_ir_val(idx, Loc::Reg(Rdi), env);
                self.emit_instrs([
                    Instr::Sar(BinArgs::ToReg(Rdi, Arg32::Imm(1))),
                    Instr::Mov(MovArgs::ToMem(mref![Rax + 8 * Rdi + 16], Reg32::Reg(Rcx)))
                ]);
            },
            IRExpr::VecGet(v, ix) => {
                self.compile_ir_val(v, Loc::Reg(Rax), env);
                self.compile_ir_val(ix, Loc::Reg(Rcx), env);
                self.emit_instrs([
                    Instr::Sar(BinArgs::ToReg(Rcx, Arg32::Imm(1))),
                    Instr::Sub(BinArgs::ToReg(Rax, Arg32::Imm(1))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Mem(mref![Rax + 8 * Rcx + 16]))),
                ]);
            },
            IRExpr::VecLen(v) => {
                self.compile_ir_val(v, Loc::Reg(Rax), env);
                self.emit_instrs([
                    Instr::Sub(BinArgs::ToReg(Rax, Arg32::Imm(1))),
                    Instr::Mov(MovArgs::ToReg(Rax, Arg64::Mem(mref![Rax + 8]))),
                    Instr::Sal(BinArgs::ToReg(Rax, Arg32::Imm(1)))
                ]);
            },
            IRExpr::Val(v) => self.compile_ir_val(v, Loc::Reg(Rax), env),
            IRExpr::PrintStack => {
                self.emit_instrs([
                    Instr::Mov(MovArgs::ToReg(Rdi, Arg64::Reg(STACK_BASE))),
                    Instr::Mov(MovArgs::ToReg(Rsi, Arg64::Reg(Rbp))),
                    Instr::Mov(MovArgs::ToReg(Rdx, Arg64::Reg(Rsp))),
                    Instr::Call("snek_print_stack".to_string()),
                ]);
            },
            IRExpr::Gc => {
                self.emit_instrs([
                    Instr::Mov(MovArgs::ToReg(Rdi, Arg64::Reg(HEAP_PTR))),
                    Instr::Mov(MovArgs::ToReg(Rsi, Arg64::Reg(STACK_BASE))),
                    Instr::Mov(MovArgs::ToReg(Rdx, Arg64::Reg(Rbp))),
                    Instr::Mov(MovArgs::ToReg(Rcx, Arg64::Reg(Rsp))),
                    Instr::Call("snek_gc".to_string()),
                ]);
            },
        }
    }

    /// target is assumed to be a *register*
    fn compile_ir_val(&mut self, v : &Val, target: Loc, env: &mut MutableMap<Symbol, i32>){
        match v {
            Val::Num(n) => self.move_to(target, Arg64::Imm(*n << 1)),//format!("mov {target}, {}", *n << 1),
            Val::True => self.move_to(target, Arg64::Imm(7)),//format!("mov {target}, 7"),
            Val::False => self.move_to(target, Arg64::Imm(3)),//format!("mov {target}, 3"),
            Val::Input => self.move_to(target, Arg32::Reg(INPUT_REG)),//format!("mov {target}, rdi"),
            Val::Nil => self.move_to(target, Arg64::Imm(1)),//format!("mov {target}, 1"),
            Val::Var(x) => {
                let offset = match env.get(x) {
                    Some(offset) => -(*offset) * 8,
                    None => {
                        panic!("Unbound identifier {x}")
                    }
                };
                self.move_to(target, Arg64::Mem(MemRef { reg: Rbp, offset: Offset::Constant(offset) }));
                //format!("mov {target}, [rsp - {offset}] ; {x}")
            }
        }
    }

    fn compile_ir_var(&mut self, var: Symbol, target: Loc, env: &mut MutableMap<Symbol, i32>){
        let offset = match env.get(&var) {
            Some(offset) => -(*offset) * 8,
            None => {
                panic!("Unbound identifier {var}")
            }
        };
        self.move_to(target, Arg64::Mem(MemRef { reg: Rbp, offset: Offset::Constant(offset) }));
    }

    fn move_to(&mut self, dst: Loc, src: impl Into<Arg64>) {
        let src = src.into();
        if dst == src {
            return;
        }
        match (dst, src) {
            (Loc::Reg(reg), _) => self.emit_instr(Instr::Mov(MovArgs::ToReg(reg, src))),
            (Loc::Mem(dst), Arg64::Reg(src)) => {
                self.emit_instr(Instr::Mov(MovArgs::ToMem(dst, Reg32::Reg(src))))
            }
            (Loc::Mem(dst), Arg64::Imm(src)) => {
                if let Ok(src) = src.try_into() {
                    self.emit_instr(Instr::Mov(MovArgs::ToMem(dst, Reg32::Imm(src))))
                } else {
                    self.emit_instrs([
                        Instr::Mov(MovArgs::ToReg(CHECK_REG, Arg64::Imm(src))),
                        Instr::Mov(MovArgs::ToMem(dst, Reg32::Reg(CHECK_REG))),
                    ])
                }
            }
            (Loc::Mem(dst), Arg64::Mem(src)) => self.emit_instrs([
                Instr::Mov(MovArgs::ToReg(CHECK_REG, Arg64::Mem(src))),
                Instr::Mov(MovArgs::ToMem(dst, Reg32::Reg(CHECK_REG))),
            ]),
        }
    }

    fn memset(&mut self, start: u32, count: u32, elem: Reg32) {
        for mem in locals(start, count) {
            self.emit_instr(Instr::Mov(MovArgs::ToMem(mem, elem)));
        }
    }

    fn emit_instrs(&mut self, instrs: impl IntoIterator<Item = Instr>) {
        self.instrs.extend(instrs);
    }

    fn emit_instr(&mut self, instr: Instr) {
        self.instrs.push(instr)
    }
    fn next_tag(&mut self) -> u32 {
        self.tag = self.tag.checked_add(1).unwrap();
        self.tag - 1
    }
}

// /// target is assumed to be a *register*
// fn compile_ir_val(v : &Val, target: &str, env: &mut MutableMap<Symbol, i32>) -> String {
//     match v {
//         Val::Num(n) => format!("mov {target}, {}", *n << 1),
//         Val::True => format!("mov {target}, 7"),
//         Val::False => format!("mov {target}, 3"),
//         Val::Input => format!("mov {target}, rdi"),
//         Val::Nil => format!("mov {target}, 1"),
//         Val::Var(x) => {
//             let offset = match env.get(x) {
//                 Some(offset) => (*offset) * 8,
//                 None => {
//                     panic!("Unbound identifier {x}")
//                 }
//             };
//             format!("mov {target}, [rsp - {offset}] ; {x}")
//         }
//     }
// }

// fn compile_ir_expr(e : &IRExpr, env: &mut MutableMap<Symbol, i32>) -> String {
//     match e {
//         IRExpr::Add1(e) => {
//             let e_is = compile_ir_val(&e, "rax", env);
//             format!(
//                 "
//                 {e_is}
//                 add rax, 2
//             "
//             )
//         }
//         IRExpr::Sub1(e) => {
//             let e_is = compile_ir_val(&e, "rax", env);
//             format!(
//                 "
//                 {e_is}
//                 sub rax, 2
//             "
//             )
//         }
//         IRExpr::Plus(e1, e2) => {
//             let e1_is = compile_ir_val(&e1, "rax", env);
//             let e2_is = compile_ir_val(&e2, "rbx", env);
//             format!(
//                 "
//                 {e1_is}
//                 {e2_is}
//                 add rax, rbx
//             "
//             )
//         }
//         IRExpr::Minus(e1, e2) => {
//             let e1_is = compile_ir_val(&e1, "rax", env);
//             let e2_is = compile_ir_val(&e2, "rbx", env);
//             format!(
//                 "
//                 {e1_is}
//                 {e2_is}
//                 sub rax, rbx
//             "
//             )
//         }
//         IRExpr::Times(e1, e2) => {
//             let e1_is = compile_ir_val(&e1, "rax", env);
//             let e2_is = compile_ir_val(&e2, "rbx", env);
//             format!(
//                 "
//                 {e1_is}
//                 {e2_is}
//                 sar rax, 1
//                 imul rax, rbx
//             "
//             )
//         }
//         IRExpr::Divide(e1, e2) => {
//             let e1_is = compile_ir_val(&e1, "rax", env);
//             let e2_is = compile_ir_val(&e2, "rbx", env);
//             format!(
//                 "
//                 {e1_is}
//                 {e2_is}
//                 cqo
//                 idiv rbx
//                 sal rax, 1
//             "
//             )
//         }
//         IRExpr::Eq(e1, e2) => {
//             let e1_is = compile_ir_val(&e1, "rax", env);
//             let e2_is = compile_ir_val(&e2, "rbx", env);
//             format!(
//                 "
//                 {e1_is}
//                 {e2_is}
//                 cmp rax, rbx
//                 mov rbx, 7
//                 mov rax, 3
//                 cmove rax, rbx
//             "
//             )
//         }
        
//         IRExpr::Lt(e1, e2) => {
//             let e1_is = compile_ir_val(&e1, "rax", env);
//             let e2_is = compile_ir_val(&e2, "rbx", env);
//             format!(
//                 "
//                 {e1_is}
//                 {e2_is}
//                 cmp rax, rbx
//                 mov rbx, 7
//                 mov rax, 3
//                 cmovl rax, rbx
//             "
//             )
//         }
//         IRExpr::Print(v) => {
//             let v_is = compile_ir_val(&v, "rax", env);
//             let offset = env.len() * 8;
//             format!(
//                 "
//                 {v_is}
//                 sub rsp, {offset}
//                 push rdi
//                 mov rdi, rax
//                 call snek_print
//                 pop rdi
//                 add rsp, {offset}
//             "
//             ) 
//         }
//         IRExpr::Call1(f, arg) => {
//             let arg_is = compile_ir_val(&arg, "rax", env);
//             let offset = env.len() * 8;
//             format!(
//                 "
//                 {arg_is}
//                 sub rsp, {offset}
//                 push rdi
//                 push rax
//                 call {f}
//                 add rsp, 8
//                 pop rdi
//                 add rsp, {offset}
//             "
//             )
//         }
//         IRExpr::Call2(f, arg1, arg2) => {
//             let arg1_is = compile_ir_val(&arg1, "rax", env);
//             let arg2_is = compile_ir_val(&arg2, "rbx", env);
//             let offset = env.len() * 8;
//             format!(
//                 "
//                 {arg1_is}
//                 {arg2_is}
//                 sub rsp, {offset}
//                 push rdi
//                 push rbx
//                 push rax
//                 call {f}
//                 add rsp, 16
//                 pop rdi
//                 add rsp, {offset}
//             "
//             )
//         }
//         IRExpr::Pair(e1, e2) => {
//             let e1_is = compile_ir_val(&e1, "rax", env);
//             let e2_is = compile_ir_val(&e2, "rbx", env);
//             format!(
//                 "
//                 {e1_is}
//                 {e2_is}
//                 mov [r15], rax
//                 mov [r15+8], rbx
//                 mov rax, r15
//                 add rax, 1
//                 add r15, 16
//             "
//             )
//         }
//         IRExpr::Fst(e) => {
//             let e_is = compile_ir_val(&e, "rax", env);
//             format!(
//                 "
//                 {e_is}
//                 mov rax, [rax-1]
//             "
//             )
//         }
//         IRExpr::Snd(e) => {
//             let e_is = compile_ir_val(&e, "rax", env);
//             format!(
//                 "
//                 {e_is}
//                 mov rax, [rax+7]
//             "
//             )
//         }
//         IRExpr::SetFst(p, v) => {
//             let p_is = compile_ir_val(&p, "rax", env);
//             let v_is = compile_ir_val(&v, "rbx", env);
//             format!(
//                 "
//                 {p_is}
//                 {v_is}
//                 mov [rax-1], rbx
//                 mov rax, rbx
//             "
//             )
//         }
//         IRExpr::SetSnd(p, v) => {
//             let p_is = compile_ir_val(&p, "rax", env);
//             let v_is = compile_ir_val(&v, "rbx", env);
//             format!(
//                 "
//                 {p_is}
//                 {v_is}
//                 mov [rax+7], rbx
//                 mov rax, rbx
//             "
//             )
//         }
//         IRExpr::Val(v) => compile_ir_val(v, "rax", env)
//     }
// }

// fn compile_ir_step(s : &Step, env: &mut MutableMap<Symbol, i32>, lbl : &Symbol) -> String {
//     match s {
//         Step::Label(l) => format!("{lbl}_{l}:\n"),
//         Step::If(v, thn, els) => {
//             let v_is = compile_ir_val(&v, "rax", env);
//             format!(
//                 "
//                 {v_is}
//                 cmp rax, 3
//                 je {lbl}_{els}
//                 jmp {lbl}_{thn}
//             "
//             )
//         }
//         Step::Goto(l) => format!("jmp {lbl}_{l}\n"),
//         Step::Do(e) => compile_ir_expr(e, env),
//         Step::Set(x, e) => {
//             let e_is = compile_ir_expr(e, env);
//             let offset = match env.get(x) {
//                 Some(offset) => (*offset) * 8,
//                 None => {
//                     panic!("Unbound identifier {x}")
//                 }
//             };
//             format!(
//                 "
//                 {e_is}
//                 mov [rsp - {offset}], rax    ; {x}
//             "
//             )
//         }
//     }
// }

// /// The argument for env is deliberately a *mutable* hashmap, so that we can
// /// incrementally add bindings. There is no depth, so in some ways we've made
// /// things _worse_ until we can register allocate reasonably.
// fn compile_ir_block(b : &Block, env: &mut MutableMap<Symbol, i32>, lbl: &Symbol) -> String {
//     let mut steps: String = String::new();
//     for step in &b.steps {
//         steps.push_str(&compile_ir_step(&step, env, lbl));
//     }
//     steps
// }

// fn compile_ir_def(d : &Def, ctx: &IRCtx) -> String {
//     let mut env = calc_env(&d.body);
//     for i in 0..d.args.len() {
//         env.insert(d.args[i].clone(), (-1)-(i as i32));
//     }
//     let body_is = compile_ir_block(&d.body, &mut env, &d.name);
//     let name = d.name.to_string();
//     return format!(
//         "
//         {name}:
//         {body_is}
//         ret
//     "
//     );
// }
fn raise_undefined_fun(fun: Symbol) {
    panic!("function {fun} not defined")
}

fn raise_wrong_number_of_args(fun: Symbol, expected: usize, got: usize) {
    panic!("function {fun} takes {expected} arguments but {got} were supplied")
}

// fn calc_env(b : &Block) -> MutableMap<Symbol, i32> {
//     let mut env = MutableMap::new();
//     for step in &b.steps {
//         match step {
//             Step::Set(x, _) => {
//                 if !env.contains_key(x) {
//                     let offset = env.len() as i32;
//                     env.insert(x.clone(), offset + 1);
//                 }
//             }
//             _ => {}
//         }
//     }
//     env
// }
fn locals(start: u32, count: u32) -> impl Iterator<Item = MemRef> {
    (start..start + count).map(|i| mref![Rbp - %(8 * (i + 1))])
}

// pub fn compile_ir_program(p : &Prog) -> (String, String) {
//     let mut ctx = IRCtx{
//         env: MutableMap::new(),
//         funs: MutableMap::new(),
//     };
//     for def in &p.defs[..] {
//         ctx.funs.insert(def.name, def.args.len());
//     }
//     let mut defs: String = String::new();
//     for def in &p.defs[..] {
//         defs.push_str(&compile_ir_def(&def, &ctx));
//     }
//     let mut main_env = calc_env(&p.main);
//     (defs, compile_ir_block(&p.main, &mut main_env, &Symbol::new("main")))
// }