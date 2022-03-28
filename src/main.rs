mod bfjit;
use crate::bfjit::{compile_bf, jit_bf};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Brainfuck JIT compiler")]
struct Args {
    #[clap(short, long, help = "Input file for the compiler.")]
    infile: String,

    #[clap(
        short,
        long,
        default_value_t = 30000,
        help = "Tape length for the compiler"
    )]
    tape_len: usize,
    
    #[clap(short, long, help="Compile Brainfuck to an object or assembly file")]
    outfile: Option<String>,

    #[clap(short, long, help="Compile Brainfuck to an assembly file")]
    asm: bool,

    #[clap(long, help="Print LLVM IR generated from the input to stderr.")]
    ir: bool

    
}


fn main() -> Result<(), Box<dyn Error>> {
    
    let args = Args::parse();
    let infile = Path::new(args.infile.as_str());
    let tape_len = args.tape_len;

    let mut in_buffer = String::new();
    File::open(infile)?.read_to_string(&mut in_buffer)?;
    if let Some(outfile) = args.outfile {
        let outfile = Path::new(outfile.as_str());
        compile_bf(in_buffer.as_str(), tape_len, outfile, args.asm, args.ir)?;
    }
    else {
        jit_bf(in_buffer.as_str(), tape_len, args.ir)?;
    }
    Ok(())
}
