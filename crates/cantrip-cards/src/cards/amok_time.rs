use cantrip_api::*;
use cantrip_mechanics::{effects, fight::targets_fight, targets};
use cantrip_oracle as oracle;

/// I: "Target creature you control fights up to one target creature an opponent controls." (CR 701.14a). If either
/// chosen target is illegal as this chapter resolves, neither creature fights (CR 701.14b, 608.2b).
fn fight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    targets_fight(g, item, 0, 1);
    Ok(())
}

/// II: "You gain 3 life." (CR 119.3).
fn gain_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 3)
}

/// III: "Return target creature card from your graveyard to your hand." An illegal target is not returned (CR 608.2b).
fn return_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_first_target_to_hand(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, fight, "I \u{2014} Target creature you control fights up to one target creature an opponent controls.")
            .targets(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS.optional()]),
        chapter(2, gain_life, "II \u{2014} You gain 3 life."),
        chapter(3, return_creature, "III \u{2014} Return target creature card from your graveyard to your hand.")
            .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD]),
    ],
    ..oracle::AMOK_TIME
};
