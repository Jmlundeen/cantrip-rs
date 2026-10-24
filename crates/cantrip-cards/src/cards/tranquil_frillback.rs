use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

const PAY: Question = Question::new(
    "PayForModes",
    "pay {G} up to three times?",
    "When you do, choose up to that many: destroy target artifact or enchantment, exile target player's graveyard, you gain 4 life",
    "Pay {cost}",
    "Don't pay",
)
.no_default();

/// "When this creature enters, you may pay {G} up to three times. When you pay this cost one or more times, choose up to
/// that many": one payment whose number of times is chosen as it is paid (CR 118.12, 608.2d), and a reflexive ability that
/// triggers once for any number of times above none, counting them (CR 603.12, 603.12a).
fn enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let paid = g.may_pay_times(you, item.object, item.object, mana("{G}"), 3, &PAY)?;
    if paid > 0 {
        let this = g.card_id(&CARD);
        g.reflexive_trigger_counted(item.object, you, AbilityRef::delayed(this, 0), Target::None, paid);
    }
    Ok(())
}

/// "Destroy target artifact or enchantment." (CR 701.8.)
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// The reflexive ability's modes, as many as {G} was paid times at most (CR 603.12a, 700.2b), chosen with their targets as
/// it is put on the stack (CR 603.3c).
static MODAL: ModalDef = modes(&[
    mode(destroy, "Destroy target artifact or enchantment.").targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    mode(effects::exile_first_target_players_graveyard, "Exile target player's graveyard.").targets(&[targets::PLAYER]),
    mode(|g, item| effects::gain_life(g, item, 4), "You gain 4 life."),
])
.up_to_that_many();

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, enters, "When this creature enters, you may pay {G} up to three times. When you pay this cost one or more times, choose up to that many \u{2014} Destroy target artifact or enchantment; or exile target player's graveyard; or you gain 4 life.")],
    delayed: &[modal_delayed_trigger(&MODAL, "When you pay this cost one or more times, choose up to that many \u{2014} Destroy target artifact or enchantment; or exile target player's graveyard; or you gain 4 life.")],
    ..oracle::TRANQUIL_FRILLBACK
};
