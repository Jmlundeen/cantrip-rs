use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

/// "Target legendary creature gains menace until end of turn" (CR 611.2a, 702.111): the ability's one target. A target no longer
/// legal as it resolves gains nothing (CR 608.2b).
fn target_gains_menace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Menace);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::ANY_COLOR_FOR_ONE,
        ability(cost("{1}, {T}"), target_gains_menace, "Smear Campaign \u{2014} {1}, {T}: Target legendary creature gains menace until end of turn. Activate only as a sorcery.")
            .sorcery_speed()
            .targets(&[targets::LEGENDARY_CREATURE]),
    ],
    ..oracle::DAILY_BUGLE_BUILDING
};
