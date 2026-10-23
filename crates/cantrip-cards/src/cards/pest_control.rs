use cantrip_api::*;
use cantrip_mechanics::effects::destroy_nonland_permanents_with_mana_value_at_most;
use cantrip_mechanics::triggers::cycling;
use cantrip_oracle as oracle;

/// "Destroy all nonland permanents with mana value 1 or less." They are destroyed all at once (CR 701.8a).
fn destroy_cheap_nonland_permanents(g: &mut dyn Game, _item: &StackItem) -> Result<(), Suspend> {
    destroy_nonland_permanents_with_mana_value_at_most(g, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_cheap_nonland_permanents),
    abilities: &[cycling(mana("{2}"), "Cycling {2} ({2}, Discard this card: Draw a card.)")],
    ..oracle::PEST_CONTROL
};
