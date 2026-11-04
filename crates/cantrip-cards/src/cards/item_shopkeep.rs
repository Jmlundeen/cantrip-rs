use cantrip_api::*;
use cantrip_mechanics::triggers::YOU_ATTACK;
use cantrip_mechanics::{attach, effects, targets};
use cantrip_oracle as oracle;

/// "Target attacking equipped creature" (CR 115.1, 506.4, 301.5a): a creature attacking now with an Equipment attached to it, whoever
/// controls either; judged when the ability is put on the stack and again as it resolves (CR 608.2b), so a creature that stops
/// attacking makes it illegal, though one that is merely unequipped by then still gets menace (the ruling).
fn attacking_equipped_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::attacking_creature_filter(q, asker, t) && matches!(t, Target::Object(id) if attach::is_equipped(q, id))
}

/// "Target attacking equipped creature gains menace until end of turn" (CR 702.110, 611.2a): nothing if the target is no longer
/// legal as the ability resolves (CR 608.2b).
fn gains_menace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Menace);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOU_ATTACK,
        gains_menace,
        "Whenever you attack, target attacking equipped creature gains menace until end of turn.",
    )
    .targets(&[target(attacking_equipped_creature)])],
    ..oracle::ITEM_SHOPKEEP
};
