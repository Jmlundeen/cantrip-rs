use cantrip_api::*;
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control. It gains lifelink until end
/// of turn." (CR 122.1a, 702.15, 611.2a): two sentences, so the counter and the keyword are separate instructions (CR 608.2c).
fn bless(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    g.end_sentence();
    effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Lifelink);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        BEGINNING_OF_YOUR_COMBAT,
        bless,
        "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control. It gains lifelink until end of turn.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::ROSA_RESOLUTE_WHITE_MAGE
};
