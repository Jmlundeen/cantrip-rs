use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, exile_target_until_leaves_effect, targets};
use cantrip_oracle as oracle;

/// "Target nonland permanent an opponent controls with mana value 3 or less" (CR 115.1, 202.3: an {X} permanent's X is 0,
/// a token's mana value is 0).
fn nonland_permanent_of_an_opponent_with_mana_value_3_or_less(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) <= 3)
}

/// "Discard a legendary card": a card in the activator's hand with the legendary supertype.
fn legendary_card(q: &dyn Query, _: &Asker, card: ObjectId) -> bool {
    q.chars(card).supertypes.legendary
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_target_until_leaves_effect, "When this artifact enters, exile target nonland permanent an opponent controls with mana value 3 or less until this artifact leaves the battlefield.")
            .targets(&[target(nonland_permanent_of_an_opponent_with_mana_value_3_or_less)]),
        ability(cost("{1}, {T}").and(discard(1, legendary_card)), |g, item| effects::draw(g, item, 1), "{1}, {T}, Discard a legendary card: Draw a card."),
    ],
    ..oracle::SOLITARY_CELL
};
