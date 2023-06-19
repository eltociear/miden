use super::{
    parse_checked_param,
    AdviceInjectorNode::*,
    Instruction::AdvInject,
    Node::{self, Instruction},
    ParsingError, Token,
};

// INSTRUCTION PARSERS
// ================================================================================================

/// Returns `AdvInject` instruction node with an appropriate internal advice injector variant.
///
/// # Errors
/// Returns an error if parsing of the internal advice injector variant fails due to wrong number
/// of parameters or invalid parameter values.
pub fn parse_adv_inject(op: &Token) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "adv");
    if op.num_parts() < 2 {
        return Err(ParsingError::missing_param(op));
    }

    let injector = match op.parts()[1] {
        "push_u64div" => match op.num_parts() {
            2 => AdvInject(PushU64div),
            _ => return Err(ParsingError::extra_param(op)),
        },
        "push_ext2intt" => match op.num_parts() {
            2 => AdvInject(PushExt2intt),
            _ => return Err(ParsingError::extra_param(op)),
        },
        "push_smtget" => match op.num_parts() {
            2 => AdvInject(PushSmtGet),
            _ => return Err(ParsingError::extra_param(op)),
        },
        "push_mapval" => match op.num_parts() {
            2 => AdvInject(PushMapVal),
            _ => return Err(ParsingError::extra_param(op)),
        },
        "push_mtnode" => match op.num_parts() {
            2 => AdvInject(PushMtNode),
            _ => return Err(ParsingError::extra_param(op)),
        },
        "insert_mem" => match op.num_parts() {
            2 => AdvInject(InsertMem),
            _ => return Err(ParsingError::extra_param(op)),
        },
        "insert_hdword" => match op.num_parts() {
            2 => AdvInject(InsertHdword { domain: 0 }),
            3 => {
                let domain = parse_checked_param::<u8, _>(op, 2, 0..=u8::MAX)?;
                AdvInject(InsertHdword { domain })
            }
            _ => return Err(ParsingError::extra_param(op)),
        },
        _ => return Err(ParsingError::invalid_op(op)),
    };

    Ok(Instruction(injector))
}
