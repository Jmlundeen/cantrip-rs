use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_mechanics::statics::{all_creatures, have_haste};
use cantrip_oracle as oracle;

/// "Whenever a player taps a basic land for mana" (CR 106.12a, 605.1b): any player, a land that has the basic supertype
/// (CR 205.4c) as it is tapped, that produced some mana.
fn basic_land_tapped_for_mana(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    let (land, player, produced) = mana::land_tapped_for_mana(q, event)?;
    if !q.chars(land).supertypes.basic {
        return None;
    }
    mana::that_land_produced(land, player, produced)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::TAPPED, basic_land_tapped_for_mana),
        mana::add_one_mana_of_any_type_that_land_produced,
        "Whenever a player taps a basic land for mana, that player adds one mana of any type that land produced.",
    )
    .mana_ability()],
    statics: &[static_ability(Layer::Ability, all_creatures, have_haste, "All creatures have haste.")],
    ..oracle::LAVALEAPER
};
