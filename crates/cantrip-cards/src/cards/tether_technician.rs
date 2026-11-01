use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, you may discard a card. When you do, ..." (CR 701.9, 603.12):
/// the optional discard happens on resolution; only then is the reflexive damage ability created.
fn discard_for_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::may_discard_a_card(g, item)? {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, discard_for_damage, "When this creature enters, you may discard a card. When you do, this creature deals 2 damage to any target.")],
    delayed: &[delayed_trigger(effects::damage_first_target::<2>, "When you do, this creature deals 2 damage to any target.")
        .targets(&[targets::ANY])],
    ..oracle::TETHER_TECHNICIAN
};
