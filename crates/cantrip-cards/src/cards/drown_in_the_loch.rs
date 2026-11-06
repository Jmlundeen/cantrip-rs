use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
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

static MODES: ModalDef = modes(&[
    mode(effects::counter_first_target, "Counter target spell with mana value less than or equal to the number of cards in its controller's graveyard.")
        .targets(&[target(spell_with_mana_value_at_most_graveyard)]),
    mode(effects::destroy_first_target, "Destroy target creature with mana value less than or equal to the number of cards in its controller's graveyard.")
        .targets(&[target(creature_with_mana_value_at_most_graveyard)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODES), ..oracle::DROWN_IN_THE_LOCH };
