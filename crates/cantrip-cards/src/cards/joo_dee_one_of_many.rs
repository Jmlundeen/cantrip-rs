use cantrip_api::*;
use cantrip_mechanics::{copy::create_token_copy_of_it, effects, select};
use cantrip_oracle as oracle;

/// "Surveil 1. Create a token that's a copy of this creature, then sacrifice an artifact or creature" (CR 701.25a, 707.2, 701.21a):
/// the steps in this order as one resolution (CR 608.2c). The copy is made first, so it is among the permanents the controller
/// must sacrifice from, and is the only choice left when nothing else is an artifact or creature (the ruling); the sacrifice is
/// not optional, so a choice is asked of the controller as soon as there is more than one.
fn surveil_copy_and_sacrifice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 1)?;
    g.end_sentence();
    create_token_copy_of_it(g, item, |_, values| values)?;
    g.end_sentence();
    effects::may_sacrifice(g, item.controller, item.object, select::artifact_or_creature_permanents(select::exactly(1)))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{B}, {T}"),
        surveil_copy_and_sacrifice,
        "{B}, {T}: Surveil 1. Create a token that's a copy of this creature, then sacrifice an artifact or creature. Activate only as a sorcery. (To surveil 1, look at the top card of your library. You may put it into your graveyard.)",
    )
    .sorcery_speed()],
    ..oracle::JOO_DEE_ONE_OF_MANY
};
