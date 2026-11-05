use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "{T}: This creature deals 1 damage to each opponent." (CR 120.3): one damage event for every opponent, all together.
fn ping_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, ping_each_opponent, "{T}: This creature deals 1 damage to each opponent."),
        triggered(triggers::NONCREATURE_SPELL_YOU_CAST, effects::untap_self, "Whenever you cast a noncreature spell, untap this creature."),
    ],
    ..oracle::TAL_SHIAR_ANALYST
};
