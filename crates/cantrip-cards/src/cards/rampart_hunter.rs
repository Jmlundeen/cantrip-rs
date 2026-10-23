use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::{pump, targets, triggers};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+2 and gains deathtouch until end of turn" (CR 611.2c, 702.2, layer 6 and 7c): the
/// ability's target, if it is still legal as the ability resolves (CR 608.2b).
fn hunt(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        instruction(g, |g| {
            pump(g, item.object, creature, 2, 2);
            effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Deathtouch);
        });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, hunt, "When this creature enters, target creature gets +2/+2 and gains deathtouch until end of turn.").targets(&[targets::CREATURE])],
    ..oracle::RAMPART_HUNTER
};
