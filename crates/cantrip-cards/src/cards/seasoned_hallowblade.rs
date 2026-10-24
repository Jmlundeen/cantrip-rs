use cantrip_api::*;
use cantrip_mechanics::effects::gain_indestructible_until_end_of_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(discard_cards(1), tap_then_gain_indestructible, "Discard a card: Tap this creature. It gains indestructible until end of turn.")],
    ..oracle::SEASONED_HALLOWBLADE
};

/// "Tap this creature. It gains indestructible until end of turn." (CR 608.2c): the tap first, then the keyword, each
/// sentence its own instruction. If the creature has left the battlefield it is a new object (CR 400.7) and neither happens.
fn tap_then_gain_indestructible(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creature = item.object;
    if targets::permanent(g, creature) {
        g.tap(creature);
        g.end_sentence();
        gain_indestructible_until_end_of_turn(g, creature, creature);
    }
    Ok(())
}
