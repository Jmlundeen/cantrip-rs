use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets, triggers, untap};
use cantrip_oracle as oracle;

/// "When this Aura enters, untap up to five lands." (CR 701.26a): any lands, chosen as it resolves and not targeted (the
/// card's ruling), zero to five of them.
fn untap_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    untap::up_to_lands(g, item.object, item.controller, 5)
}

/// "Enchant creature" (CR 303.4a) and "You control enchanted creature." (CR 613.1b, layer 2), at the Aura's timestamp (CR
/// 613.7e), for as long as it is attached.
pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(triggers::THIS_ENTERS, untap_lands, "When this Aura enters, untap up to five lands.")],
    statics: &[control_changing(attached_object, "You control enchanted creature.")],
    ..oracle::TREACHERY
};
