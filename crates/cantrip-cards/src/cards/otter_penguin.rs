use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "This creature gets +1/+2 until end of turn and can't be blocked this turn" (CR 611.2a, 509.1b): two effects of one instruction on the
/// same creature.
fn pump_and_unblockable(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.one_instruction(&mut |g| {
        effects::this_gets::<1, 2>(g, item)?;
        effects::this_cant_be_blocked_this_turn(g, item)
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_DRAW_YOUR_SECOND_CARD,
        pump_and_unblockable,
        "Whenever you draw your second card each turn, this creature gets +1/+2 until end of turn and can't be blocked this turn.",
    )],
    ..oracle::OTTER_PENGUIN
};
