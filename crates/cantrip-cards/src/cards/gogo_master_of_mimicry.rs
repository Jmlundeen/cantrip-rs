use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Copy target activated or triggered ability you control X times. You may choose new targets for the copies." (CR 707.10,
/// 707.10c) One copy at a time, each with its own chance at new targets; each keeps the original's X, modes and division, and
/// is neither activated nor triggered. Its only target no longer legal, the ability does not resolve at all (CR 608.2b).
fn copy_x_times(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for _ in 0..item.x {
        g.copy_ability(item.target(0), item.controller, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{X}{X}, {T}"),
        copy_x_times,
        "{X}{X}, {T}: Copy target activated or triggered ability you control X times. You may choose new targets for the copies. This ability can't be copied and X can't be 0.",
    )
    .targets(&[targets::YOUR_ACTIVATED_OR_TRIGGERED_ABILITY])
    .x_cant_be_zero()
    .cant_be_copied()],
    ..oracle::GOGO_MASTER_OF_MIMICRY
};
