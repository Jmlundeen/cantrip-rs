use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ATTACKS, THIS_ENTERS};
use cantrip_mechanics::{effects, tokens};
use cantrip_oracle as oracle;

/// "Create two 1/1 colorless Myr artifact creature tokens" (CR 701.7): the two are made together.
fn two_myr(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::MYR, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, two_myr, "When Memnarch enters, create two 1/1 colorless Myr artifact creature tokens."),
        triggered(THIS_ATTACKS, effects::draw_a_card_for_each_artifact_you_control, "Whenever Memnarch attacks, draw a card for each artifact you control."),
    ],
    ..oracle::MEMNARCH_THE_WARDEN
};

pub static EXTRA: &[&CardDef] = &[&tokens::MYR];
