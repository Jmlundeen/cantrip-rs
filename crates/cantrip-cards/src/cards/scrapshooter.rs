use cantrip_api::*;
use cantrip_mechanics::{destroy_target, gift, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Destroy target artifact or enchantment an opponent controls" (CR 701.8).
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Gift a card" (CR 702.174e): the permanent's gift triggered ability gives it when it enters (CR 702.174b), and the
/// "if the gift was promised" of its own ability is the same intervening "if" (CR 603.4).
pub static CARD: CardDef = CardDef {
    gift: Some(gift::CARD),
    abilities: &[
        gift::trigger("When this creature enters, if its gift cost was paid, the chosen opponent draws a card."),
        triggered(THIS_ENTERS, destroy, "When this creature enters, if the gift was promised, destroy target artifact or enchantment an opponent controls.")
            .condition(gift::paid)
            .targets(&[targets::ARTIFACT_OR_ENCHANTMENT_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::SCRAPSHOOTER
};
