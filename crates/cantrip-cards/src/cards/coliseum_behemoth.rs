use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Draw a card." (CR 121.1)
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

/// "When this creature enters, choose one" (CR 603.3c, 700.2): the mode, and the target of the destroy mode, are chosen as the
/// ability is put on the stack. The target is any artifact or enchantment, its own or an opponent's (CR 701.8).
static MODAL: ModalDef = modes(&[
    mode(effects::destroy_first_target, "Destroy target artifact or enchantment.").targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    mode(draw_a_card, "Draw a card."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(THIS_ENTERS, &MODAL, "When this creature enters, choose one \u{2014} \u{2022} Destroy target artifact or enchantment. \u{2022} Draw a card.")],
    ..oracle::COLISEUM_BEHEMOTH
};
