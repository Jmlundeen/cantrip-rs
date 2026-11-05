use cantrip_api::*;
use cantrip_mechanics::{damage_target, effects, lands, lands_you_control, permanents_with_subtype_you_control, targets, triggers};
use cantrip_oracle as oracle;

/// "When Omnath enters, it deals damage to any target equal to the number of Elementals you control." (CR 608.2, 120.3).
fn deals_damage_any_target_equal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let elemental = g.words(item.object).subtype(Subtype::Elemental);
    let amount = permanents_with_subtype_you_control(g, item.controller, elemental) as u32;
    damage_target(g, item, 0, amount);
    Ok(())
}

/// "Landfall — Whenever a land you control enters, put a +1/+1 counter on target Elemental you control. If you control eight
/// or more lands, draw a card." (CR 608.2, 121.1, 122.1).
fn landfall_counter_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_target(g, item)?;
    g.end_sentence();
    if lands_you_control(g, item.controller) >= 8 {
        g.draw(item.controller, 1);
        g.end_sentence();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            deals_damage_any_target_equal,
            "When Omnath enters, it deals damage to any target equal to the number of Elementals you control.",
        )
        .targets(&[targets::ANY]),
        triggered(
            triggers::LANDFALL,
            landfall_counter_and_draw,
            "Landfall — Whenever a land you control enters, put a +1/+1 counter on target Elemental you control. If you control eight or more lands, draw a card.",
        )
        .targets(&[target(targets::permanent_of_subtype_you_control::<{ lands::sub(Subtype::Elemental) }>)]),
    ],
    ..oracle::OMNATH_LOCUS_OF_THE_ROIL
};
