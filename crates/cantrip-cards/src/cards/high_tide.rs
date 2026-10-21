use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

/// "Until end of turn, whenever a player taps an Island for mana, that player adds an additional {U}."
/// The spell creates a delayed triggered ability (CR 603.7b) that is a mana ability (CR 605.1b), so each one resolves at
/// once, as the land's mana ability does (CR 605.4a). It watches players, not lands: an Island that enters later counts too,
/// and any land with the type Island, whoever controls it, gives the extra mana to the player who tapped it, not to this
/// spell's controller.
fn watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::delayed::this_turn(g, item, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], watch),
    delayed: &[triggered(
        when_asked(EventMask::TAPPED, mana::a_player_taps_a_land_type_for_mana::<{ lands::sub(Subtype::Island) }>),
        mana::that_player_adds_an_additional::<{ Mana::U as u8 }>,
        "Until end of turn, whenever a player taps an Island for mana, that player adds an additional {U}.",
    )
    .mana_ability()],
    ..oracle::HIGH_TIDE
};
