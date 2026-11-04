use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{discard_two_unless_discarding, statics};
use cantrip_oracle as oracle;

/// "Draw two cards. Then discard two cards unless you discard an artifact card." The player decides as the discard is made
/// (CR 608.2d) between one artifact card and two cards of any kind; with no artifact card in hand the choice is not offered.
/// Two cards are chosen freely, so an artifact card may be one of them (its ruling).
fn enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
    discard_two_unless_discarding(g, &[you], item.object, |chars| chars.is(CardType::Artifact))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, enters, "When Alpharael enters, draw two cards. Then discard two cards unless you discard an artifact card.")],
    statics: &[static_ability(Layer::Ability, statics::itself_during_your_turn, statics::have_deathtouch, "During your turn, Alpharael has deathtouch.")],
    labels: &["Discard an artifact card", "Discard two cards"],
    ..oracle::ALPHARAEL_DREAMING_ACOLYTE
};
