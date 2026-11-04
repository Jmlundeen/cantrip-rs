use cantrip_api::*;
use cantrip_mechanics::targets::{self, card_in_your_graveyard};
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_oracle as oracle;

/// "Creature or Spacecraft card with mana value 5 or less from your graveyard" (CR 115.1, 202.3, 205.3g): a creature card, or
/// a card with the Spacecraft subtype (a Spacecraft card is an artifact, never a creature, in the graveyard). X in its mana
/// cost is 0 there (CR 202.3e), which `Query::mana_value` answers.
fn creature_or_spacecraft_card_with_mana_value_5_or_less_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_your_graveyard(q, asker, t, &[])
        && matches!(t, Target::Object(id) if q.mana_value(id) <= 5 && {
            let c = q.chars(id);
            c.is(CardType::Creature) || c.has_subtype(Subtype::Spacecraft)
        })
}

/// Returns the target to the battlefield under the spell's controller (CR 400.3), then creates a Lander token (CR 111.10u) -
/// the second sentence is "Then", so it follows the return. With its one target illegal the spell does not resolve at all
/// (CR 608.2b), so no token is made.
fn return_it_then_create_a_lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::return_first_target_to_battlefield(g, item)?;
    tokens::create_lander(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(creature_or_spacecraft_card_with_mana_value_5_or_less_in_your_graveyard)], return_it_then_create_a_lander),
    // As an additional cost to cast this spell, sacrifice an artifact or creature (CR 118.8, 601.2b).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::artifact_or_creature)),
    ..oracle::SCROUNGE_FOR_ETERNITY
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
