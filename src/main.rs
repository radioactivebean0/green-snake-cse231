use std::{
    env,
    fs::File,
    io::{self, Read, Write},
};
// use anf::*;
// use ir::*;
// use ircompiler::*;
// use iroptimizer::*;

mod asm;
mod compiler;
mod parser;
mod syntax;
mod anf;
mod ir;
mod ircompiler;
mod iroptimizer;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let in_name = &args[1];
    let out_name = &args[2];
    let mut in_contents = String::new();
    let mut in_file = File::open(in_name)?;
    in_file.read_to_string(&mut in_contents)?;
    let expr = parser::parse(&in_contents);
    let anf_prog = anf::anf_program(&expr);
    let anf_str_prog = anf::flatprogram_to_string(&anf_prog);
    let ir_prog = ir::anf_to_ir(&anf_prog);
    let ir_str_prog = ir::ir_to_string(&ir_prog);
    let opt_ir_prog = iroptimizer::optimize_ir(&ir_prog);
    //print!("OPTIMIZED");
    let opt_ir_asm = ircompiler::compile_ir_prog(&opt_ir_prog);

    // let ir_asm = ircompiler::compile_ir_prog(&ir_prog);
    let asm = compiler::compile(&expr);
    // if args.len() < 4 || &args[3] == "--og" {
    //     let mut out_file = File::create(out_name)?;
    //     out_file.write_all(asm.as_bytes())?;
    // } else if &args[3] == "--ir" {
    //     let mut out_file = File::create(out_name)?;
    //     out_file.write_all(ir_asm.as_bytes())?;
    // } else if &args[3] == "--opt"{
        let mut out_file = File::create(out_name)?;
        out_file.write_all(opt_ir_asm.as_bytes())?;
    // } else {
        // let mut out_file = File::create(out_name)?;
        // out_file.write_all(asm.as_bytes())?;
    // }

    let mut ir_outfile = File::create(out_name.to_owned()+".ir")?;
    ir_outfile.write_all(ir_str_prog.as_bytes())?;
    let mut anf_outfile = File::create(out_name.to_owned()+".anf")?;
    anf_outfile.write_all(anf_str_prog.as_bytes())?;
    Ok(())
}
