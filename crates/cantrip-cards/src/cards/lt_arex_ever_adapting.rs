use cantrip_api::*;
use cantrip_mechanics::hand;
use cantrip_mechanics::lands;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When Lt. Arex enters, draw three cards. You may put up to three land cards from your hand onto the battlefield
/// tapped." (CR 121.1, 608.2c, 608.2d): the draw ends the sentence, so the cards just drawn are in the hand to choose
/// from; then up to three land cards of the hand are chosen together and put tapped (CR 101.4), or none. Putting a land
/// onto the battlefield is not playing it, so it uses no land play (CR 305.2).
fn enters_draw_then_may_put_lands_tapped(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    g.end_sentence();
    hand::may_put_up_to_onto_battlefield_tapped(g, item.controller, item.object, 3, |g, c| g.chars(c).is(CardType::Land))?;
    Ok(())
}

/// "Lt. Arex's power and toughness are each equal to the number of differently named lands you control." (CR 604.3,
/// 613.4a): characteristic-defining, so it applies in every zone and reads current land names through the layers. Captain
/// Pike, Fullness of Time prints the same ability.
pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(
        Layer::PtDefine,
        lands::power_and_toughness_are_differently_named_lands_you_control,
        "Lt. Arex's power and toughness are each equal to the number of differently named lands you control.",
    )
    .lost_pt(0, 0)],
    abilities: &[triggered(
        THIS_ENTERS,
        enters_draw_then_may_put_lands_tapped,
        "When Lt. Arex enters, draw three cards. You may put up to three land cards from your hand onto the battlefield tapped.",
    )],
    ..oracle::LT_AREX_EVER_ADAPTING
};
