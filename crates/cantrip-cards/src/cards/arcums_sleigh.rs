use cantrip_api::*;
use cantrip_mechanics::{combat, effects, lands, targets};
use cantrip_oracle as oracle;

/// "{2}, {T}: Target creature gains vigilance until end of turn. Activate only during combat and only if defending player controls a snow land." (CR 608.2, 611.2a).
fn target_creature_gains_vigilance_activate(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Vigilance);
    Ok(())
}

/// "{2}, {T}: Target creature gains vigilance until end of turn. Activate only during combat and only if defending player controls a snow land." (CR 602.5).
fn target_creature_gains_vigilance_activate_if(q: &dyn Query, _: &Asker) -> bool {
    combat::during_combat(q)
        && q.state().opponents(q.state().turn.active).any(|defending| lands::controls_snow_land(q, defending))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{2}").and(TAP),
                target_creature_gains_vigilance_activate,
                "{2}, {T}: Target creature gains vigilance until end of turn. Activate only during combat and only if defending player controls a snow land.",
            )
            .targets(&[targets::CREATURE])
            .condition(target_creature_gains_vigilance_activate_if),
    ],
    ..oracle::ARCUMS_SLEIGH
};
