use cantrip_api::*;
use cantrip_mechanics::delayed;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Create a Lander token. At the beginning of the end step on your next turn, sacrifice that token." (CR 111.10u, 603.7):
/// the delayed ability carries every token created, however many a replacement made of it (the ruling: Exalted Sunborn's
/// extra Landers are sacrificed too), and sacrifices those still on the battlefield under its controller's control.
fn make_a_lander_until_your_next_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let created = tokens::create_by_rule(g, item.controller, &LANDER, 1, false)?;
    delayed::at_step_on_your_next_turn_about(g, item, Step::End, 0, &created);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        make_a_lander_until_your_next_turn,
        "When this creature enters, create a Lander token. At the beginning of the end step on your next turn, sacrifice that token.",
    )],
    delayed: &[delayed_trigger(delayed::sacrifice_created, "At the beginning of the end step on your next turn, sacrifice that token.")],
    ..oracle::KAV_LANDSEEKER
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
