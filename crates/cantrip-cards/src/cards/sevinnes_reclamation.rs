use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::flashback;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target permanent card with mana value 3 or less from your graveyard."
fn permanent_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(
        q,
        asker,
        t,
        &[CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Land, CardType::Planeswalker, CardType::Battle],
    ) && matches!(t, Target::Object(id) if q.mana_value(id) <= 3)
}

/// Return the target to the battlefield; then, if this spell was cast from a graveyard (any way), the controller may
/// copy it and choose a new target for the copy. A copy was never cast (CR 707.10), so it does not copy itself again.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    if g.obj(item.object).cast_from == Some(ZoneKind::Graveyard)
        && effects::ask_may(g, item.controller, item.object, Purpose::Optional)?
    {
        g.copy_spell(item.object, item.controller, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(permanent_card_in_your_graveyard)], effect),
    ways: &[flashback::way(cost("{4}{W}"), "Flashback {4}{W}")],
    ..oracle::SEVINNES_RECLAMATION
};
