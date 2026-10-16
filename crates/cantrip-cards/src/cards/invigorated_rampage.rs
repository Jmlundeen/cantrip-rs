use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +4/+0 and gains trample until end of turn."
fn one_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, 4, 0);
        cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

/// "Two target creatures each get +2/+0 and gain trample until end of turn." Exactly two are chosen (CR 601.2c); each one still
/// a legal target as the spell resolves gets it (CR 608.2b).
fn two_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for slot in 0..item.target_count as usize {
        if let Some(creature) = targets::object(g, item, slot) {
            pump(g, item.object, creature, 2, 0);
            cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
        }
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(one_creature, "Target creature gets +4/+0 and gains trample until end of turn.").targets(&[targets::CREATURE]),
    mode(two_creatures, "Two target creatures each get +2/+0 and gain trample until end of turn.")
        .targets(&[targets::CREATURE.between(2, 2)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::INVIGORATED_RAMPAGE };
