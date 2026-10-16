use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets, triggers::cycling};
use cantrip_oracle as oracle;

/// Target creature gains trample and haste until end of turn: the granted ability's source is the land (CR 113.7).
fn trample_and_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
        cantrip_mechanics::haste_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::LAND),
    abilities: &[cycling(mana("{1}"), "Cycling {1} ({1}, Discard this card: Draw a card.)")],
    delayed: &[ability(TAP, trample_and_haste, "{T}: Target creature gains trample and haste until end of turn.")
        .targets(&[targets::CREATURE])],
    statics: &[static_ability(Layer::Ability, attached_object, cantrip_mechanics::statics::grant_delayed_ability, "Enchanted land has \"{T}: Target creature gains trample and haste until end of turn.\"")],
    ..oracle::FOOTFALL_CRATER
};
