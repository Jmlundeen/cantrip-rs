use cantrip_api::*;
use cantrip_mechanics::library::search_aura_or_equipment_to_hand;
use cantrip_mechanics::statics;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Search your library for an Aura or Equipment card, reveal it, put it into your hand, then shuffle."
fn tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_aura_or_equipment_to_hand(g, item.controller, item.object)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, statics::itself_during_your_turn, statics::have_flying, "During your turn, Sun-Spider has flying.")],
    abilities: &[triggered(THIS_ENTERS, tutor, "When Sun-Spider enters, search your library for an Aura or Equipment card, reveal it, put it into your hand, then shuffle.")],
    ..oracle::SUN_SPIDER_NIMBLE_WEBBER
};
