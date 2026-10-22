use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_mechanics::typecycling;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

/// "Another target creature you control gets +2/+0 and gains trample until end of turn."
fn rally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        instruction(g, |g| {
            pump(g, item.object, creature, 2, 0);
            cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
        });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ATTACKS, rally, "Whenever Oliphaunt attacks, another target creature you control gets +2/+0 and gains trample until end of turn.")
            .targets(&[targets::OTHER_CREATURE_YOU_CONTROL]),
        typecycling::ability(Subtype::Mountain, mana("{1}"), "Mountaincycling {1} ({1}, Discard this card: Search your library for a Mountain card, reveal it, put it into your hand, then shuffle.)"),
    ],
    ..oracle::OLIPHAUNT
};
