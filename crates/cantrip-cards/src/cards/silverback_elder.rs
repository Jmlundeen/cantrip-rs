use cantrip_api::*;
use cantrip_mechanics::{effects, library, targets};
use cantrip_mechanics::triggers::CREATURE_SPELL_YOU_CAST;
use cantrip_oracle as oracle;

/// Mode 1: "Destroy target artifact or enchantment." (CR 701.8)
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// Mode 2: "Look at the top five cards of your library. You may put a land card from among them onto the battlefield
/// tapped. Put the rest on the bottom of your library in a random order."
fn look_for_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_onto_battlefield(g, item.controller, item.object, 5, 1, "land", |q, c| q.chars(c).is(CardType::Land), None, true, library::Rest::BottomRandom)?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy, "Destroy target artifact or enchantment.").targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    mode(look_for_a_land, "Look at the top five cards of your library. You may put a land card from among them onto the battlefield tapped. Put the rest on the bottom of your library in a random order."),
    mode(|g, item| effects::gain_life(g, item, 4), "You gain 4 life."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(CREATURE_SPELL_YOU_CAST, &MODAL, "Whenever you cast a creature spell, choose one \u{2014} Destroy target artifact or enchantment; or look at the top five cards of your library, you may put a land card from among them onto the battlefield tapped, and put the rest on the bottom of your library in a random order; or you gain 4 life.")],
    ..oracle::SILVERBACK_ELDER
};
