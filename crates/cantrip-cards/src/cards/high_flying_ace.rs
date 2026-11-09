use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature without flying gains flying until end of turn."
fn gain_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Flying);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}{W}"), gain_flying, "{3}{W}: Target creature without flying gains flying until end of turn. Activate only as a sorcery.")
        .targets(&[targets::CREATURE_WITHOUT_FLYING])
        .sorcery_speed()],
    ..oracle::HIGH_FLYING_ACE
};
