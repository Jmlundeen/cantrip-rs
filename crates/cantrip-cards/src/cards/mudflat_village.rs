use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target Bat, Lizard, Rat, or Squirrel card from your graveyard": a card of any type in its controller's graveyard with
/// one of those creature types, the words read through the land's own (CR 612).
fn critter_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    if !targets::card_in_your_graveyard(q, asker, t, &[]) {
        return false;
    }
    let words = q.words(asker.source);
    let chars = q.chars(id);
    [Subtype::Bat, Subtype::Lizard, Subtype::Rat, Subtype::Squirrel].into_iter().any(|s| chars.has_subtype(words.subtype(s)))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add(Mana::B).only_for(ManaRestriction::CREATURE_SPELLS), "{T}: Add {B}. Spend this mana only to cast a creature spell."),
        ability(
            cost("{1}{B}, {T}").and(SACRIFICE_THIS),
            effects::return_first_target_to_hand,
            "{1}{B}, {T}, Sacrifice this land: Return target Bat, Lizard, Rat, or Squirrel card from your graveyard to your hand.",
        )
        .targets(&[target(critter_card_in_your_graveyard)]),
    ],
    ..oracle::MUDFLAT_VILLAGE
};
