use cantrip_api::*;
use cantrip_mechanics::{choose, destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target artifact" / "Destroy target enchantment" (CR 701.8): nothing if the target is no longer legal (CR 608.2b); an
/// indestructible one stays (the verb's).
fn destroy_its_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Draw a card. Then you may put a land card from your hand onto the battlefield tapped." (CR 121.1, 608.2d, 110.2a): the whole
/// hand is shown, the card just drawn included, and any land card in it may be chosen, or none; nothing is asked with no land card.
/// Putting a land onto the battlefield is not playing it, so it uses no land play (CR 305.2).
fn draw_and_put_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 1);
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let lands: Vec<ObjectId> = hand.iter().copied().filter(|&c| g.chars(c).is(CardType::Land)).collect();
    let Some(&land) = choose::cards_from(g, you, item.object, Purpose::PutOntoBattlefield, hand, lands, 1, true)?.first() else {
        return Ok(());
    };
    g.put_all_onto_battlefield_as(&[(land, you, true)])?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy_its_target, "Destroy target artifact.").targets(&[targets::ARTIFACT]),
    mode(destroy_its_target, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
    mode(draw_and_put_a_land, "Draw a card. Then you may put a land card from your hand onto the battlefield tapped."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_ability(
        cost("{2}").and(SACRIFICE_THIS),
        &MODAL,
        "{2}, Sacrifice this creature: Choose one \u{2014} destroy target artifact; destroy target enchantment; or draw a card, then you may put a land card from your hand onto the battlefield tapped.",
    )],
    ..oracle::INSIDIOUS_FUNGUS
};
