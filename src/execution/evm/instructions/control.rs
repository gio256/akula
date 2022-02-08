use crate::execution::evm::{interpreter::JumpdestMap, state::*, StatusCode};
use bytes::Bytes;
use ethnum::U256;

#[inline(always)]
pub(crate) fn ret(
    mut state: ExecutionState,
    gasometer: &mut Gasometer,
    output_data: &mut Bytes,
) -> Result<(), StatusCode> {
    let offset = *state.stack.get(0);
    let size = *state.stack.get(1);

    if let Some(region) = super::memory::get_memory_region(&mut state, gasometer, offset, size)? {
        *output_data = state
            .memory
            .0
            .freeze()
            .slice(region.offset..region.offset + region.size.get());
    }

    Ok(())
}

#[inline(always)]
pub(crate) fn op_jump(
    state: &mut ExecutionState,
    jumpdest_map: &JumpdestMap,
) -> Result<usize, StatusCode> {
    let dst = state.stack.pop();
    if !jumpdest_map.contains(dst) {
        return Err(StatusCode::BadJumpDestination);
    }

    Ok(dst.as_usize())
}

#[inline(always)]
pub(crate) fn calldataload(state: &mut ExecutionState) {
    let index = state.stack.pop();

    let input_len = state.message.input_data.len();

    state.stack.push({
        if index > u128::try_from(input_len).unwrap() {
            U256::ZERO
        } else {
            let index_usize = index.as_usize();
            let end = core::cmp::min(index_usize + 32, input_len);

            let mut data = [0; 32];
            data[..end - index_usize].copy_from_slice(&state.message.input_data[index_usize..end]);

            U256::from_be_bytes(data)
        }
    });
}

#[inline(always)]
pub(crate) fn calldatasize(state: &mut ExecutionState) {
    state.stack.push(
        u128::try_from(state.message.input_data.len())
            .unwrap()
            .into(),
    );
}
