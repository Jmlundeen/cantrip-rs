use cantrip_api::*;
use cantrip_mechanics::effects::return_first_target_to_hand;
use cantrip_mechanics::{cost_reduction, lands, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target Villain card from your graveyard" (CR 115.1, 205.3), read through the words of this creature's text (CR 612): a card in its
/// controller's graveyard that is a Villain there.
fn villain_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[])
        && matches!(t, Target::Object(id) if q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Villain)))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, return_first_target_to_hand, "When Tombstone enters, return target Villain card from your graveyard to your hand.")
        .targets(&[target(villain_card_in_your_graveyard)])],
    cost_changes: &[cost_reduction::subtype_spells_you_cast::<1, { lands::sub(Subtype::Villain) }>],
    ..oracle::TOMBSTONE_CAREER_CRIMINAL
};
