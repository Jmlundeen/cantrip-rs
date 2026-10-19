use cantrip_api::*;
use cantrip_mechanics::library::search_aura_or_equipment_to_hand;
use cantrip_oracle as oracle;

/// Search your library for an Aura or Equipment card, reveal it, put it into your hand, then shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_aura_or_equipment_to_hand(g, item.controller, item.object)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::OPEN_THE_ARMORY };
