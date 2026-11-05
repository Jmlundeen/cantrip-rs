use cantrip_api::*;
use cantrip_mechanics::{destroy_target, exile_target, targets};
use cantrip_oracle as oracle;

/// "Alter the Past — Exile target card from a graveyard. You gain 2 life." A spell whose only target is illegal does not
/// resolve, so a card that became an illegal target is not exiled and no life is gained (CR 608.2b).
fn alter_the_past(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if exile_target(g, item, 0) {
        g.gain_life(item.controller, 2);
    }
    Ok(())
}

/// "Target artifact or enchantment with mana value 3 or less" (CR 202.3): both types, and the mana value in the zone the
/// permanent is in.
fn artifact_or_enchantment_mv3(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::ARTIFACT_OR_ENCHANTMENT.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) <= 3)
}

/// "Obliterate the Transwarp Network — Destroy target artifact or enchantment with mana value 3 or less." "Destroyed"
/// carries regeneration and indestructible with it (CR 701.8a-b).
fn obliterate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Choose one —" (CR 700.2), the modes in printed order.
static MODAL: ModalDef = modes(&[
    mode(alter_the_past, "Alter the Past — Exile target card from a graveyard. You gain 2 life.")
        .targets(&[targets::CARD_IN_A_GRAVEYARD]),
    mode(obliterate, "Obliterate the Transwarp Network — Destroy target artifact or enchantment with mana value 3 or less.")
        .targets(&[target(artifact_or_enchantment_mv3)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::TEST_OF_COURAGE };
