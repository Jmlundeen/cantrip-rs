use cantrip_api::*;
use cantrip_mechanics::{effects, flashback, targets};
use cantrip_oracle as oracle;

/// "Sacrifice a nonland permanent": any nonland permanent the caster controls.
fn nonland_permanent_to_sacrifice(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    !q.chars(id).is(CardType::Land)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], effects::exile_first_target),
    // As an additional cost to cast this spell, sacrifice a nonland permanent (CR 118.8), with either way of casting it.
    additional_cost: Some(Cost::sacrifice(1, nonland_permanent_to_sacrifice)),
    ways: &[flashback::way(cost("{2}{W}{B}"), "Flashback {2}{W}{B}")],
    ..oracle::RITE_OF_OBLIVION
};
