use cantrip_api::*;
use cantrip_mechanics::{assimilate, effects, targets, triggers};
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Assimilate a creature card exiled with it": the linked pile the entry trigger filled, one card chosen as this
/// resolves, put onto the battlefield with a +1/+1 counter as a Borg artifact creature (CR 122.6, 205.1b, 614.12).
fn assimilate_one_exiled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    assimilate::a_creature_card_exiled_with(g, item)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            effects::exile_first_target_players_graveyard,
            "When this creature enters, exile target opponent's graveyard.",
        )
        .targets(&[targets::OPPONENT]),
        triggered(
            THIS_ATTACKS,
            assimilate_one_exiled,
            "Whenever this creature attacks, assimilate a creature card exiled with it.",
        ),
    ],
    ..oracle::ASSIMILATION_ASSESSOR
};
