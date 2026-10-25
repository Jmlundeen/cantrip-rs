use cantrip_api::*;
use cantrip_mechanics::{effects, empower, triggers};
use cantrip_oracle as oracle;

/// "When Way of the Paradox enters, empower Jace 5" (CR 701.71).
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 5)
}

/// "Whenever you activate a loyalty ability, you gain 1 life. You may play an additional land this turn." One trigger, two
/// sentences of one resolution (CR 608.2c, 603.8): the life, then the permission (CR 305.2), which lasts until end of turn.
fn gain_life_and_play_an_additional_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 1)?;
    g.end_sentence();
    g.additional_land_plays(item, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, empower_jace, "When Way of the Paradox enters, empower Jace 5."),
        triggered(
            triggers::YOU_ACTIVATE_A_LOYALTY_ABILITY,
            gain_life_and_play_an_additional_land,
            "Whenever you activate a loyalty ability, you gain 1 life. You may play an additional land this turn.",
        ),
    ],
    ..oracle::WAY_OF_THE_PARADOX
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
