use std::ffi::CStr;
use std::process::exit;
use std::ptr::null_mut;

use llvm::core::{LLVMBuildRet, LLVMDisposeBuilder};
use llvm::target::{LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllDisassemblers};
use llvm::target_machine::LLVMGetHostCPUFeatures;
use llvm::{
    core::{
        LLVMAddFunction, LLVMAppendBasicBlock, LLVMBuildAlloca, LLVMBuildCall2, LLVMBuildStore,
        LLVMConstInt, LLVMCreateBuilder, LLVMDisposeMessage, LLVMDisposeModule, LLVMFunctionType,
        LLVMGetNamedFunction, LLVMInt32Type, LLVMInt64Type, LLVMInt8Type, LLVMModuleCreateWithName,
        LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMSetDataLayout, LLVMSetTarget,
    },
    target::{
        LLVMCopyStringRepOfTargetData, LLVM_InitializeAllAsmParsers, LLVM_InitializeAllTargetInfos,
        LLVM_InitializeAllTargetMCs, LLVM_InitializeAllTargets,
    },
    target_machine::{
        LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetDataLayout, LLVMCreateTargetMachine,
        LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetTargetFromTriple,
        LLVMTargetMachineEmitToFile, LLVMTargetRef,
    },
    LLVMType,
};

use crate::parser::Parser;

extern crate llvm_sys as llvm;

pub struct Codegen<'c> {
    parser: Parser<'c>,
    module: *mut llvm::LLVMModule,
    builder: *mut llvm::LLVMBuilder,
}

macro_rules! cstr {
    ($s: literal) => {
        $s.as_ptr() as *mut i8
    };
}

impl<'c> Codegen<'c> {
    pub unsafe fn new(parser: Parser<'c>) -> Self {
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmParsers();
        LLVM_InitializeAllAsmPrinters();
        LLVM_InitializeAllDisassemblers();

        let module = LLVMModuleCreateWithName(cstr!("main\0"));

        let builder = LLVMCreateBuilder();

        let empty_array: [*mut LLVMType; 0] = [];

        let main = LLVMAddFunction(
            module,
            cstr!("main\0"),
            LLVMFunctionType(LLVMInt32Type(), empty_array.as_ptr() as *mut _, 0, 0),
        );

        LLVMPositionBuilderAtEnd(builder, LLVMAppendBasicBlock(main, cstr!("\0")));

        let r#i8 = LLVMInt8Type();
        let r#i64 = LLVMInt64Type();
        let i8ptr = LLVMPointerType(r#i8, 0);

        let data = LLVMBuildAlloca(builder, i8ptr, cstr!("data\0"));
        let ptr = LLVMBuildAlloca(builder, i8ptr, cstr!("ptr\0"));

        let calloc = LLVMGetNamedFunction(module, cstr!("calloc"));

        let data_ptr = LLVMBuildCall2(
            builder,
            i8ptr,
            calloc,
            [LLVMConstInt(r#i64, 30000, 0), LLVMConstInt(r#i64, 1, 0)].as_ptr() as *mut _,
            2,
            cstr!(""),
        );

        LLVMBuildStore(builder, data_ptr, data);
        LLVMBuildStore(builder, data_ptr, ptr);

        LLVMBuildRet(builder, LLVMConstInt(LLVMInt32Type(), 0, 1));

        Self {
            parser,
            module,
            builder,
        }
    }

    pub unsafe fn build(&self) {
        let target_triple = LLVMGetDefaultTargetTriple();

        let mut target: LLVMTargetRef = null_mut();
        let mut error: *mut i8 = null_mut();

        if LLVMGetTargetFromTriple(target_triple, &mut target, &mut error) != 0 {
            println!("{}", CStr::from_ptr(error).to_str().unwrap());
            LLVMDisposeMessage(error);
            exit(1);
        }

        let target_machine = LLVMCreateTargetMachine(
            target,
            target_triple,
            cstr!("generic\0"),
            LLVMGetHostCPUFeatures(),
            LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            llvm::target_machine::LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        LLVMSetTarget(self.module, target_triple);
        LLVMSetDataLayout(
            self.module,
            LLVMCopyStringRepOfTargetData(LLVMCreateTargetDataLayout(target_machine)),
        );

        if LLVMTargetMachineEmitToFile(
            target_machine,
            self.module,
            cstr!("output.o\0"),
            llvm::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
            &mut error,
        ) != 0
        {
            println!("{}", CStr::from_ptr(error).to_str().unwrap());
            LLVMDisposeMessage(error);
            exit(1);
        }

        LLVMDisposeTargetMachine(target_machine);
        LLVMDisposeBuilder(self.builder);
        LLVMDisposeModule(self.module);
    }
}
