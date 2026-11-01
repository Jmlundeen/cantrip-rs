use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets, triggers};
use cantrip_oracle as oracle;

/// "{1}{G}, Discard this card: Return target land card from your graveyard to the battlefield tapped." Activated from
/// the hand, the zone its cost moves it out of (CR 113.6m), the discard being part of the activation cost (CR 602.1a).
fn return_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::return_target_land_card_from_graveyard_tapped(g, item, 0)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_BECOMES_THE_TARGET_OF_AN_OPPONENTS_SPELL_OR_ABILITY,
            |g, item| effects::draw(g, item, 1),
            "Whenever Altanak becomes the target of a spell or ability an opponent controls, draw a card.",
        ),
        ability(
            cost("{1}{G}").and(DISCARD_THIS),
            return_land,
            "{1}{G}, Discard this card: Return target land card from your graveyard to the battlefield tapped.",
        )
        .targets(&[targets::LAND_CARD_IN_YOUR_GRAVEYARD])
        .from(ZoneKind::Hand),
    ],
    ..oracle::ALTANAK_THE_THRICE_CALLED
};
