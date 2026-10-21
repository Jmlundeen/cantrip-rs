use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::TAPPED, mana::a_player_taps_a_land_for_mana),
        mana::add_one_mana_of_any_type_that_land_produced,
        "Whenever a player taps a land for mana, that player adds one mana of any type that land produced.",
    )
    .mana_ability(TriggeredMana::OneOfProduced)],
    ..oracle::MANA_FLARE
};
