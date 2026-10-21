use cantrip_api::*;
use cantrip_oracle as oracle;

fn legendary_creature_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    cantrip_mechanics::targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature])
        && matches!(t, Target::Object(id) if q.chars(id).supertypes.legendary)
}

/// Return target legendary creature card from your graveyard to the battlefield. That creature gains haste. Exile it at the
/// beginning of the next end step. Spliced onto an Arcane spell (CR 702.47c) its target slot is numbered after that spell's
/// own slots (`StackItem::spec_targets`).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = g.card_id(&CARD);
    let spell_card = g.obj(item.object).card;
    let own = if spell_card == me { 0 } else { g.card(spell_card).spell.map_or(0, |s| s.targets.len()) };
    let Some(slot) = g.state().spec_targets(item, own).next() else { return Ok(()) };
    if !g.target_legal(item, slot) {
        return Ok(());
    }
    let Target::Object(card) = g.state().target(item, slot) else { return Ok(()) };
    let controller = item.controller;
    let creature = g.put_onto_battlefield(card, controller, false)?;
    if creature != card || g.obj(creature).zone.is(ZoneKind::Battlefield) {
        cantrip_mechanics::effects::gain_haste(g, item.object, creature);
        g.delayed_trigger(
            DelayedWhen::Step { step: Step::End, on_turn_of: None },
            item.object,
            controller,
            AbilityRef::delayed(me, 0),
            Target::Object(creature),
        );
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(legendary_creature_card_in_your_graveyard)], effect),
    splice: Some(mana("{2}{B}")),
    delayed: &[delayed_trigger(cantrip_mechanics::delayed::exile_subject, "Exile it at the beginning of the next end step.")],
    ..oracle::GORYOS_VENGEANCE
};
