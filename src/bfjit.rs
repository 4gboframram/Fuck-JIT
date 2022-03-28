use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::{Linkage, Module};
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::OptimizationLevel;
use inkwell::{AddressSpace, IntPredicate};
use itertools::Itertools;
use std::error::Error;
use std::path::Path;
type MainJitType = unsafe extern "C" fn(*mut u8);

#[derive(Debug)]
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: Option<ExecutionEngine<'ctx>>,
    tape_ptr_ptr: PointerValue<'ctx>,
    entry_block: BasicBlock<'ctx>,
    main: FunctionValue<'ctx>,
    jump_stack: Vec<Loop<'ctx>>,
    putchar_fn: FunctionValue<'ctx>,
    getchar_fn: FunctionValue<'ctx>,
}

#[derive(Debug)]
pub struct Loop<'ctx> {
    start: BasicBlock<'ctx>,
    body: BasicBlock<'ctx>,
    exit: BasicBlock<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new_jit(context: &'ctx Context) -> Result<CodeGen<'ctx>, Box<dyn Error>> {
        let module = context.create_module("bf");
        let execution_engine =
            Some(module.create_jit_execution_engine(OptimizationLevel::Aggressive)?);
        let main_t = context.void_type().fn_type(
            &[context.i8_type().ptr_type(AddressSpace::Generic).into()],
            false,
        );
        let main = module.add_function("main", main_t, Some(Linkage::External));

        let i32_type = context.i32_type();
        let getchar_fn_type = i32_type.fn_type(&[], false);
        let getchar_fn = module.add_function("getchar", getchar_fn_type, Some(Linkage::External));

        let putchar_fn_type = i32_type.fn_type(&[i32_type.into()], false);
        let putchar_fn = module.add_function("putchar", putchar_fn_type, Some(Linkage::External));

        let entry_block = context.append_basic_block(main, "entry");
        let tape_ptr = main.get_nth_param(0).unwrap().into_pointer_value();
        let builder = context.create_builder();
        builder.position_at_end(entry_block);
        let tape_ptr_ptr = builder.build_alloca(tape_ptr.get_type(), "tape_ptr_ptr");
        builder.build_store(tape_ptr_ptr, tape_ptr);

        Ok(CodeGen {
            context,
            module,
            execution_engine,
            tape_ptr_ptr,
            entry_block,
            main,
            builder,
            getchar_fn,
            putchar_fn,
            jump_stack: Vec::new(),
        })
    }

    pub fn finalize_jit(&self) -> Option<JitFunction<MainJitType>> {
        self.builder.build_return(None);
        unsafe {
            self.execution_engine
                .as_ref()
                .unwrap()
                .get_function("main")
                .ok()
        }
    }

    fn get_ptr(&self) -> PointerValue<'ctx> {
        self.builder
            .build_load(self.tape_ptr_ptr, "tape_ptr")
            .into_pointer_value()
    }

    fn get_i8_const(&self, value: i8) -> IntValue<'ctx> {
        self.context.i8_type().const_int(value as u64, true)
    }
    fn get_i16_const(&self, value: i16) -> IntValue<'ctx> {
        self.context.i16_type().const_int(value as u64, true)
    }

    fn emit_incr_ptr(&self, cnt: i16) {
        let new_ptr = unsafe {
            self.builder
                .build_gep(self.get_ptr(), &[self.get_i16_const(cnt)], "incr_ptr")
        };
        self.builder.build_store(self.tape_ptr_ptr, new_ptr);
    }

    fn emit_incr_cell(&self, cnt: i8) {
        let curr_ptr = self.get_ptr();
        let val = self.builder.build_load(curr_ptr, "cell_val");
        let incremented_value =
            self.builder
                .build_int_add(val.into_int_value(), self.get_i8_const(cnt), "cell_incr");
        self.builder.build_store(curr_ptr, incremented_value);
    }

    fn create_compare(&self) -> IntValue<'ctx> {
        let curr_ptr = self.get_ptr();
        let val = self.builder.build_load(curr_ptr, "cell_val");

        self.builder.build_int_compare(
            IntPredicate::EQ,
            val.into_int_value(),
            self.get_i8_const(0),
            "cmp",
        )
    }

    fn emit_start_loop(&mut self) {
        let start = self.context.append_basic_block(self.main, "while_start");

        let body = self.context.append_basic_block(self.main, "while_body");
        let exit = self.context.append_basic_block(self.main, "while_exit");

        self.builder.build_unconditional_branch(start);
        self.builder.position_at_end(start);
        let cmp = self.create_compare();
        self.builder.build_conditional_branch(cmp, exit, body);

        self.jump_stack.push(Loop { start, body, exit });
        self.builder.position_at_end(body);
    }

    fn emit_end_loop(&mut self) {
        let Loop {
            start,
            body: _,
            exit,
        } = self.jump_stack.pop().unwrap();
        self.builder.build_unconditional_branch(start);
        self.builder.position_at_end(exit);
    }

    fn emit_getchar(&self) {
        let char = self
            .builder
            .build_call(self.getchar_fn, &[], "get_c")
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let ptr = self.get_ptr();
        self.builder.build_store(ptr, char);
    }

    fn emit_putchar(&self) {
        let val = self.builder.build_load(self.get_ptr(), "cell_val");
        self.builder
            .build_call(self.putchar_fn, &[val.into()], "get_c")
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
    }

    fn bf_codegen(&mut self, code: &str) {
        let insts = parse_brainfuck(code);
        insts
            .into_iter()
            .for_each(|Instruction { inst, count }| match inst {
                '+' => self.emit_incr_cell(count as i8),
                '-' => self.emit_incr_cell(-count as i8),
                '>' => self.emit_incr_ptr(count as i16),
                '<' => self.emit_incr_ptr(-count as i16),
                '[' => self.emit_start_loop(),
                ']' => self.emit_end_loop(),
                '.' => self.emit_putchar(),
                ',' => self.emit_getchar(),
                _ => {}
            });
    }

    pub fn new_comp(
        context: &'ctx Context,
        tape_length: usize,
    ) -> Result<CodeGen<'ctx>, Box<dyn Error>> {
        let module = context.create_module("bf");
        // let execution_engine = module.create_jit_execution_engine(OptimizationLevel::Aggressive)?;
        let main_t = context.i32_type().fn_type(&[], false);
        let main = module.add_function("main", main_t, Some(Linkage::External));

        let i32_type = context.i32_type();
        let getchar_fn_type = i32_type.fn_type(&[], false);
        let getchar_fn = module.add_function("getchar", getchar_fn_type, Some(Linkage::External));
        let putchar_fn_type = i32_type.fn_type(&[i32_type.into()], false);
        let putchar_fn = module.add_function("putchar", putchar_fn_type, Some(Linkage::External));

        let entry_block = context.append_basic_block(main, "entry");
        let arr_type = context.i8_type().array_type(tape_length as u32);

        let builder = context.create_builder();
        builder.position_at_end(entry_block);

        let arr_ptr = builder.build_alloca(arr_type, "tape_ptr");
        builder.build_store(arr_ptr, arr_type.const_zero());
        let tape_ptr = builder.build_bitcast(
            arr_ptr,
            context.i8_type().ptr_type(AddressSpace::Generic),
            "tape_ptr",
        );

        let tape_ptr_ptr = builder.build_alloca(tape_ptr.get_type(), "tape_ptr_ptr");
        builder.build_store(tape_ptr_ptr, tape_ptr);

        Ok(CodeGen {
            context,
            module,
            execution_engine: None,
            tape_ptr_ptr,
            entry_block,
            main,
            builder,
            getchar_fn,
            putchar_fn,
            jump_stack: Vec::new(),
        })
    }

    pub fn finalize_comp(&self) {
        self.builder
            .build_return(Some(&self.context.i32_type().const_zero()));
    }
}

pub struct Instruction {
    pub inst: char,
    pub count: isize,
}

fn parse_brainfuck(string: &str) -> Vec<Instruction> {
    string
        .chars()
        .map(|inst| Instruction { inst, count: 1 })
        .coalesce(|i1, i2| match i1.inst {
            '>' | '<' | '+' | '-' => {
                if i1.inst == i2.inst {
                    Ok(Instruction {
                        inst: i1.inst,
                        count: i1.count + i2.count,
                    })
                } else {
                    Err((i1, i2))
                }
            }
            '.' | ',' | '[' | ']' => Err((i1, i2)),
            _ => Err((
                Instruction {
                    inst: '\0',
                    count: 0,
                },
                i2,
            )),
        })
        .filter(|x| {
            x.count > 0 && {
                match x.inst {
                    '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => true,
                    _ => false, // fix bug where last character would be an instruction no matter what
                }
            }
        })
        .collect()
}

pub fn jit_bf(code: &str, tape_length: usize, print_ir: bool) -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let mut codegen = CodeGen::new_jit(&context)?;
    {
        codegen.bf_codegen(code);
        let func = codegen.finalize_jit().unwrap();
        let mut arr = vec![0u8; tape_length];
        if print_ir {
            codegen.main.print_to_stderr();
        }
        unsafe { func.call(arr[..].as_mut_ptr()) };
    };

    Ok(())
}

use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};

pub fn compile_bf(
    code: &str,
    tape_length: usize,
    outfile: &Path,
    asm: bool,
    print_ir: bool,
) -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let mut codegen = CodeGen::new_comp(&context, tape_length)?;

    Target::initialize_all(&InitializationConfig::default());
    // use the host machine as the compilation target
    let target_triple = TargetMachine::get_default_triple();
    let cpu = TargetMachine::get_host_cpu_name().to_string();
    let features = TargetMachine::get_host_cpu_features().to_string();

    // make a target from the triple
    let target = Target::from_triple(&target_triple).map_err(|e| format!("{:?}", e))?;

    // make a machine from the target
    let target_machine = target
        .create_target_machine(
            &target_triple,
            &cpu,
            &features,
            OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| "Unable to create target machine!".to_string())?;

    let filetype = if asm {
        FileType::Assembly
    } else {
        FileType::Object
    };

    codegen.bf_codegen(code);
    codegen.finalize_comp();

    if print_ir {
        codegen.main.print_to_stderr();
    }

    target_machine
        .write_to_file(&codegen.module, filetype, outfile)
        .map_err(|e| format!("{:?}", e))?;

    Ok(())
}
