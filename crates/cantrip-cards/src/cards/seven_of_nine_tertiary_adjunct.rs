use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "That Is Irrelevant — {T}: Draw a card, then discard a card. When you discard a nonland card this way, put two charge
/// counters on up to one target artifact." The discard is not optional (CR 701.9); the reflexive ability (CR 603.12)
/// triggers only when a nonland card was discarded, and its target is chosen as it goes on the stack.
fn loot_then_charge_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.card_id(&CARD);
    effects::loot_then_reflexive(g, item, AbilityRef::delayed(this, 0))
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        loot_then_charge_counters,
        "That Is Irrelevant \u{2014} {T}: Draw a card, then discard a card. When you discard a nonland card this way, put two charge counters on up to one target artifact.",
    )],
    delayed: &[delayed_trigger(
        effects::charge_counters_on_target::<2>,
        "When you discard a nonland card this way, put two charge counters on up to one target artifact.",
    )
    .targets(&[targets::ARTIFACT.optional()])],
    ..oracle::SEVEN_OF_NINE_TERTIARY_ADJUNCT
};
