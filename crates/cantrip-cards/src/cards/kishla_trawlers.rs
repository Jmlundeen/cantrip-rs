use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

const ASK: Question = Question::new(
    "Exile",
    "exile a creature card from your graveyard?",
    "When you do, return target instant or sorcery card from your graveyard to your hand",
    "Exile a creature card",
    "Don't",
);

/// "You may exile a creature card from your graveyard. When you do" (CR 701.13a, 603.12): the exile is the player's choice as the
/// ability resolves, naming a creature card, so the card is chosen as it is exiled (CR 406.3); the reflexive trigger is created only
/// if a card was actually exiled, and chooses its target as it goes on the stack. A graveyard with no creature card is not asked.
fn exile_a_creature_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if g.may_pay(you, item.object, item.object, exile_from_graveyard_of(1, filters::creature), &ASK)? {
        g.reflexive_trigger(item.object, you, AbilityRef::delayed(g.card_id(&CARD), 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, exile_a_creature_card, "When this creature enters, you may exile a creature card from your graveyard. When you do, return target instant or sorcery card from your graveyard to your hand.")],
    delayed: &[delayed_trigger(effects::return_first_target_to_hand, "When you do, return target instant or sorcery card from your graveyard to your hand.")
        .targets(&[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD])],
    ..oracle::KISHLA_TRAWLERS
};
