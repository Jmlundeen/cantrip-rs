use cantrip_api::*;
use cantrip_mechanics::{choose, effects};
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "You may tap or untap target creature" (`choose::may_tap_or_untap_target`): the target chosen as the trigger goes on the
/// stack, Tap, Untap or Do nothing as it resolves.
fn tap_or_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = g.card_id(&CARD);
    choose::may_tap_or_untap_target(g, item, card)
}

pub static CARD: CardDef = CardDef {
    labels: choose::TAP_OR_UNTAP,
    abilities: &[
        triggered(triggers::THIS_ENTERS, tap_or_untap, "When Sewer-veillance Cam enters, you may tap or untap target creature.")
            .targets(&[targets::CREATURE]),
        triggered(triggers::THIS_LEAVES, tap_or_untap, "When Sewer-veillance Cam leaves the battlefield, you may tap or untap target creature.")
            .targets(&[targets::CREATURE]),
        ability(cost("{3}{U}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 2), "{3}{U}, Sacrifice Sewer-veillance Cam: Draw two cards."),
    ],
    ..oracle::SEWER_VEILLANCE_CAM
};
