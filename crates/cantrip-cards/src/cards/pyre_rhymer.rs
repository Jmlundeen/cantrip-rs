use cantrip_api::*;
use cantrip_mechanics::{lands, mana, triggers::PROWESS};
use cantrip_oracle as oracle;

/// "Until end of turn, whenever you tap a Mountain for mana, add an additional {R}." The spell creates a delayed triggered
/// ability (CR 603.7b) that is a mana ability (CR 605.1b), so each one resolves at once, as the Mountain's mana ability does
/// (CR 605.4a). "You" is the player who controlled the spell as it resolved (CR 603.7d).
fn molten_tide(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::delayed::this_turn(g, item, item.object);
    Ok(())
}

/// Molten Tide, the prepare spell (CR 722): not a card, only ever a copy cast from exile.
pub static MOLTEN_TIDE: CardDef = CardDef {
    name: "Molten Tide",
    cost: mana("{R}"),
    types: TypeSet::of(&[CardType::Instant]),
    spell: spell(&[], molten_tide),
    delayed: &[triggered(
        when_asked(EventMask::TAPPED, mana::you_tap_a_land_type_for_mana::<{ lands::sub(Subtype::Mountain) }>),
        mana::that_player_adds_an_additional::<{ Mana::R as u8 }>,
        "Until end of turn, whenever you tap a Mountain for mana, add an additional {R}.",
    )
    .mana_ability(TriggeredMana::One(Mana::R))],
    text: "Until end of turn, whenever you tap a Mountain for mana, add an additional {R}.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[PROWESS],
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&MOLTEN_TIDE),
    ..oracle::PYRE_RHYMER
};

pub static EXTRA: &[&CardDef] = &[&MOLTEN_TIDE];
