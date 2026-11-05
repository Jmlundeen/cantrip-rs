use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "This artifact deals 2 damage to target opponent." (CR 120.2, 608.2b): the opponent takes it, not their permanents.
fn scorch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::damage_target(g, item, 0, 2);
    Ok(())
}

static TEXT: &str = "{2}, {T}, Sacrifice this artifact: Choose one \u{2014} \u{2022} This artifact deals 2 damage to target opponent. \u{2022} You gain 3 life.";

static MODES: ModalDef = modes(&[
    mode(scorch, "This artifact deals 2 damage to target opponent.").targets(&[targets::OPPONENT]),
    mode(|g, item| effects::gain_life(g, item, 3), "You gain 3 life."),
]);

/// "When this artifact enters, draw a card." (CR 603.6a, 121.1): one card for its controller.
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, draw_a_card, "When this artifact enters, draw a card."),
        modal_ability(cost("{2}, {T}").and(SACRIFICE_THIS), &MODES, TEXT),
    ],
    ..oracle::TEA_EARL_GREY_HOT
};
