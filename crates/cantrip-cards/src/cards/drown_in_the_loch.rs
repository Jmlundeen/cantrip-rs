use cantrip_api::*;
use cantrip_mechanics::{counter_target, effects, targets};
use cantrip_oracle as oracle;

/// Whether `id`'s mana value is less than or equal to the number of cards in its controller's graveyard (CR 202.3, 404.1).
fn mana_value_at_most_cards_in_its_controllers_graveyard(q: &dyn Query, id: ObjectId) -> bool {
    let cards = q.state().zone(Zone::graveyard(q.controller(id))).len() as u32;
    q.mana_value(id) <= cards
}

/// "Target spell with mana value less than or equal to the number of cards in its controller's graveyard." (CR 115.1).
fn spell_with_mana_value_at_most_graveyard(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.obj(id).zone.is(ZoneKind::Stack) && mana_value_at_most_cards_in_its_controllers_graveyard(q, id))
}

/// "Target creature with mana value less than or equal to the number of cards in its controller's graveyard." (CR 115.1).
fn creature_with_mana_value_at_most_graveyard(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.chars(id).is(CardType::Creature) && mana_value_at_most_cards_in_its_controllers_graveyard(q, id))
}

/// "Counter target spell ..." (CR 701.6a, 608.2b).
fn counter_the_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

static MODES: ModalDef = modes(&[
    mode(counter_the_spell, "Counter target spell with mana value less than or equal to the number of cards in its controller's graveyard.")
        .targets(&[target(spell_with_mana_value_at_most_graveyard)]),
    mode(effects::destroy_first_target, "Destroy target creature with mana value less than or equal to the number of cards in its controller's graveyard.")
        .targets(&[target(creature_with_mana_value_at_most_graveyard)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODES), ..oracle::DROWN_IN_THE_LOCH };
