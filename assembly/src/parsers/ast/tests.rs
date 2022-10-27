use super::{parse_module, parse_program, BTreeMap, Instruction, Node, ProcMap, ProcedureAst};
use crate::parsers::ast::{ModuleAst, ProgramAst};
use crypto::{hashers::Blake3_192, Digest, Hasher};
use vm_core::{Felt, FieldElement};

// UNIT TESTS
// ================================================================================================

/// Tests the AST parsing
#[test]
fn test_ast_parsing_program_simple() {
    let source = "begin push.0 assertz end";
    let values: Vec<Felt> = vec![Felt::ZERO];
    let nodes: Vec<Node> = vec![
        Node::Instruction(Instruction::PushConstants(values)),
        Node::Instruction(Instruction::Assertz),
    ];

    assert_program_output(source, BTreeMap::new(), nodes);
}

#[test]
fn test_ast_parsing_program_proc() {
    let source = "\
    proc.foo.1 
        loc_load.0
    end
    proc.bar.2 
        padw
    end  
    begin
        exec.foo
        exec.bar
    end";
    let proc_body1: Vec<Node> = vec![Node::Instruction(Instruction::LocLoad(Felt::ZERO))];
    let mut procedures: ProcMap = BTreeMap::new();
    procedures.insert(
        String::from("foo"),
        ProcedureAst {
            name: String::from("foo"),
            is_export: false,
            num_locals: 1,
            index: 0,
            body: proc_body1,
        },
    );
    let proc_body2: Vec<Node> = vec![Node::Instruction(Instruction::PadW)];
    procedures.insert(
        String::from("bar"),
        ProcedureAst {
            name: String::from("bar"),
            is_export: false,
            num_locals: 2,
            index: 1,
            body: proc_body2,
        },
    );
    let nodes: Vec<Node> = vec![
        Node::Instruction(Instruction::ExecLocal(0)),
        Node::Instruction(Instruction::ExecLocal(1)),
    ];
    assert_program_output(source, procedures, nodes);
}

#[test]
fn test_ast_parsing_module() {
    let source = "\
    export.foo.1 
        loc_load.0
    end";
    let mut procedures: ProcMap = BTreeMap::new();
    let proc_body: Vec<Node> = vec![Node::Instruction(Instruction::LocLoad(Felt::ZERO))];
    procedures.insert(
        String::from("foo"),
        ProcedureAst {
            name: String::from("foo"),
            is_export: true,
            num_locals: 1,
            index: 0,
            body: proc_body,
        },
    );
    parse_program(source).expect_err("Program should contain body and no export");
    let module = parse_module(source).unwrap();
    assert_eq!(module.procedures.len(), procedures.len());
    for (name, proc) in module.procedures {
        assert!(procedures.contains_key(&name));
        assert_eq!(procedures.get(&name).unwrap(), &proc);
    }
}

#[test]
fn test_ast_parsing_use() {
    let source = "\
    use.std::abc::foo
    begin
        exec.foo::bar
    end";
    let procedures: ProcMap = BTreeMap::new();
    let proc_name_digest = Blake3_192::<Felt>::hash(String::from("std::abc::foo::bar").as_bytes());
    let mut proc_name_hash = [0; 24];
    proc_name_hash.copy_from_slice(&proc_name_digest.as_bytes()[..24]);
    let nodes: Vec<Node> = vec![Node::Instruction(Instruction::ExecImported(proc_name_hash))];
    assert_program_output(source, procedures, nodes);
}

// SERIALIZATION AND DESERIALIZATION TESTS
// ================================================================================================

#[test]
fn test_ast_program_serde_simple() {
    let source = "begin push.0xabc234 push.0 assertz end";
    let program = parse_program(source).unwrap();
    let program_serialized = program.to_bytes();
    let program_deserialized = ProgramAst::from_bytes(&mut program_serialized.as_slice()).unwrap();

    assert_eq!(program, program_deserialized);
}

#[test]
fn test_ast_program_serde_local_procs() {
    let source = "\
    proc.foo.1 
        loc_load.0
    end
    proc.bar.2 
        padw
    end  
    begin
        exec.foo
        exec.bar
    end";
    let program = parse_program(source).unwrap();
    let program_serialized = program.to_bytes();
    let program_deserialized = ProgramAst::from_bytes(&mut program_serialized.as_slice()).unwrap();

    assert_eq!(program, program_deserialized);
}

#[test]
fn test_ast_program_serde_exported_procs() {
    let source = "\
    export.foo.1 
        loc_load.0
    end
    export.bar.2 
        padw
    end";
    let module = parse_module(source).unwrap();
    let module_serialized = module.to_bytes();
    let module_deserialized = ModuleAst::from_bytes(&mut module_serialized.as_slice()).unwrap();

    assert_eq!(module, module_deserialized);
}

#[test]
fn test_ast_program_serde_control_flow() {
    let source = "\
    begin
        repeat.3
            push.1
            push.0.1
        end 

        if.true
            and
            loc_store.0
        else
            padw
        end
        
        while.true
            push.5.7
            u32checked_add
            loc_store.1
            push.0
        end

        repeat.3
            push.2
            u32overflowing_mul
        end

    end";

    let program = parse_program(source).unwrap();
    let program_serialized = program.to_bytes();
    let program_deserialized = ProgramAst::from_bytes(&mut program_serialized.as_slice()).unwrap();

    assert_eq!(program, program_deserialized);
}

fn assert_program_output(source: &str, procedures: ProcMap, body: Vec<Node>) {
    let program = parse_program(source).unwrap();
    assert_eq!(program.body, body);
    assert_eq!(program.procedures.len(), procedures.len());
    for (name, proc) in program.procedures {
        assert!(procedures.contains_key(&name));
        assert_eq!(procedures.get(&name).unwrap(), &proc);
    }
}