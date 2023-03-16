use std::ffi::CStr;
use std::process::exit;
use std::ptr::null_mut;

use llvm::core::{
    LLVMAddTargetDependentFunctionAttr, LLVMBuildAdd, LLVMBuildCall2, LLVMBuildGEP2,
    LLVMBuildLoad2, LLVMBuildRet, LLVMBuildStore, LLVMDisposeBuilder, LLVMGetNamedFunction,
    LLVMPrintModuleToString,
};
use llvm::prelude::{LLVMBuilderRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef};
use llvm::target::{LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllDisassemblers};
use llvm::target_machine::LLVMGetHostCPUFeatures;
use llvm::{
    core::{
        LLVMAddFunction, LLVMAppendBasicBlock, LLVMBuildAlloca, LLVMConstInt, LLVMCreateBuilder,
        LLVMDisposeMessage, LLVMDisposeModule, LLVMFunctionType, LLVMInt32Type, LLVMInt64Type,
        LLVMInt8Type, LLVMModuleCreateWithName, LLVMPointerType, LLVMPositionBuilderAtEnd,
        LLVMSetDataLayout, LLVMSetTarget,
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

use crate::parser::{Instruction, Parser};

extern crate llvm_sys as llvm;

pub struct Codegen<'c> {
    filename: &'c str,
    parser: Parser<'c>,
    current_instruction: Instruction,
    ptr: LLVMValueRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    putchar_fn_type: LLVMTypeRef,
    getchar_fn_type: LLVMTypeRef,
}

macro_rules! cstr {
    ($s: literal) => {
        concat!($s, '\0').as_ptr() as *mut i8
    };
}

macro_rules! from_cstr {
    ($s: expr) => {
        CStr::from_ptr($s).to_str().unwrap()
    };
}

impl<'c> Codegen<'c> {
    pub unsafe fn new(filename: &'c str, mut parser: Parser<'c>) -> Self {
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmParsers();
        LLVM_InitializeAllAsmPrinters();
        LLVM_InitializeAllDisassemblers();

        let module = LLVMModuleCreateWithName(cstr!("main"));

        let builder = LLVMCreateBuilder();

        let current_instruction = parser.next().unwrap_or_default();

        let calloc_fn_type = LLVMFunctionType(
            LLVMPointerType(LLVMInt8Type(), 0),
            [LLVMInt64Type(), LLVMInt64Type()].as_ptr() as *mut _,
            2,
            0,
        );
        let calloc = LLVMAddFunction(module, cstr!("calloc"), calloc_fn_type);

        LLVMAddTargetDependentFunctionAttr(calloc, cstr!("unwind"), cstr!(""));
        LLVMAddTargetDependentFunctionAttr(calloc, cstr!("noalias"), cstr!(""));

        let putchar_fn_type =
            LLVMFunctionType(LLVMInt32Type(), [LLVMInt32Type()].as_ptr() as *mut _, 1, 0);

        LLVMAddFunction(module, cstr!("putchar"), putchar_fn_type);

        let getchar_fn_type = LLVMFunctionType(
            LLVMInt32Type(),
            ([] as [*mut LLVMType; 0]).as_ptr() as *mut _,
            0,
            0,
        );

        LLVMAddFunction(module, cstr!("getchar"), putchar_fn_type);

        let main = LLVMAddFunction(
            module,
            cstr!("main"),
            LLVMFunctionType(
                LLVMInt32Type(),
                ([] as [*mut LLVMType; 0]).as_ptr() as *mut _,
                0,
                0,
            ),
        );

        LLVMPositionBuilderAtEnd(builder, LLVMAppendBasicBlock(main, cstr!("")));

        let data = LLVMBuildAlloca(builder, LLVMPointerType(LLVMInt8Type(), 0), cstr!("data"));
        let ptr = LLVMBuildAlloca(builder, LLVMPointerType(LLVMInt8Type(), 0), cstr!("ptr"));

        let data_ptr = LLVMBuildCall2(
            builder,
            calloc_fn_type,
            calloc,
            [
                LLVMConstInt(LLVMInt64Type(), 30000, 0),
                LLVMConstInt(LLVMInt64Type(), 1, 0),
            ]
            .as_ptr() as *mut _,
            2,
            cstr!(""),
        );

        LLVMBuildStore(builder, data_ptr, data);
        LLVMBuildStore(builder, data_ptr, ptr);

        Self {
            filename,
            parser,
            current_instruction,
            ptr,
            module,
            builder,
            putchar_fn_type,
            getchar_fn_type,
        }
    }

    fn advance(&mut self) {
        self.current_instruction = self.parser.next().unwrap_or_default();
    }

    unsafe fn codegen_for_next_instruction(&mut self) {
        match self.current_instruction {
            Instruction::Output => {
                LLVMBuildCall2(
                    self.builder,
                    self.putchar_fn_type,
                    LLVMGetNamedFunction(self.module, cstr!("putchar")),
                    [LLVMBuildLoad2(
                        self.builder,
                        LLVMInt8Type(),
                        self.ptr,
                        cstr!(""),
                    )]
                    .as_ptr() as *mut _,
                    1,
                    cstr!(""),
                );
                self.advance();
            }
            Instruction::Input => {
                LLVMBuildStore(
                    self.builder,
                    LLVMBuildCall2(
                        self.builder,
                        self.getchar_fn_type,
                        LLVMGetNamedFunction(self.module, cstr!("getchar")),
                        ([] as [LLVMValueRef; 0]).as_ptr() as *mut _,
                        0,
                        cstr!(""),
                    ),
                    self.ptr,
                );
                self.advance();
            }
            Instruction::Advance | Instruction::Decrease => {
                let mut offset = if matches!(self.current_instruction, Instruction::Advance) {
                    1
                } else {
                    u64::MAX
                };

                self.advance();

                while matches!(
                    self.current_instruction,
                    Instruction::Advance | Instruction::Decrease
                ) {
                    offset += if matches!(self.current_instruction, Instruction::Advance) {
                        1
                    } else {
                        u64::MAX
                    };
                    self.advance();
                }

                if offset != 0 {
                    LLVMBuildStore(
                        self.builder,
                        LLVMBuildAdd(
                            self.builder,
                            LLVMBuildLoad2(self.builder, LLVMInt8Type(), self.ptr, cstr!("")),
                            LLVMConstInt(LLVMInt8Type(), offset, 0),
                            cstr!(""),
                        ),
                        self.ptr,
                    );
                }
            }
            Instruction::MoveRight | Instruction::MoveLeft => {
                let mut offset = if matches!(self.current_instruction, Instruction::MoveRight) {
                    1
                } else {
                    u64::MAX
                };

                self.advance();

                while matches!(
                    self.current_instruction,
                    Instruction::MoveRight | Instruction::MoveLeft
                ) {
                    offset += if matches!(self.current_instruction, Instruction::MoveRight) {
                        1
                    } else {
                        u64::MAX
                    };
                    self.advance();
                }

                if offset != 0 {
                    self.ptr = LLVMBuildGEP2(
                        self.builder,
                        LLVMPointerType(LLVMInt8Type(), 0),
                        self.ptr,
                        &mut LLVMConstInt(LLVMInt8Type(), offset, 0),
                        1,
                        cstr!("ptr"),
                    );
                }
            }
            _ => {}
        }
    }

    pub unsafe fn build(&mut self) {
        while self.current_instruction != Instruction::EndOfInput {
            self.codegen_for_next_instruction();
        }

        LLVMBuildRet(self.builder, LLVMConstInt(LLVMInt32Type(), 0, 1));

        println!("{}", from_cstr!(LLVMPrintModuleToString(self.module)));

        let target_triple = LLVMGetDefaultTargetTriple();

        let mut target: LLVMTargetRef = null_mut();
        let mut error: *mut i8 = null_mut();

        if LLVMGetTargetFromTriple(target_triple, &mut target, &mut error) != 0 {
            println!("{}", from_cstr!(error));
            LLVMDisposeMessage(error);
            exit(1);
        }

        let target_machine = LLVMCreateTargetMachine(
            target,
            target_triple,
            cstr!("generic"),
            LLVMGetHostCPUFeatures(),
            LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            llvm::target_machine::LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        LLVMSetTarget(self.module, target_triple);
        LLVMSetDataLayout(
            self.module,
            LLVMCopyStringRepOfTargetData(LLVMCreateTargetDataLayout(target_machine)),
        );

        let object_filename = format!("{}.o\0", self.filename);

        if LLVMTargetMachineEmitToFile(
            target_machine,
            self.module,
            object_filename.as_ptr() as *mut i8,
            llvm::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
            &mut error,
        ) != 0
        {
            println!("{}", from_cstr!(error));
            LLVMDisposeMessage(error);
            exit(1);
        }

        println!("Successfully compiled {}!", object_filename);

        LLVMDisposeTargetMachine(target_machine);
        LLVMDisposeBuilder(self.builder);
        LLVMDisposeModule(self.module);
    }
}
