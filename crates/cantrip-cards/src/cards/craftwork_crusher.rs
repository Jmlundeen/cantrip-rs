use cantrip_api::*;
use cantrip_mechanics::{tokens, targets};
use cantrip_oracle as oracle;

/// "This creature deals 4 damage to target creature or planeswalker." (CR 120.3a.)
fn deal_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(target) = targets::object(g, item, 0) {
        g.damage(item.object, Target::Object(target), 4);
    }
    Ok(())
}

/// "Create a 2/2 colorless Wizard Soldier creature token named Cadet." (CR 111.1.)
fn create_cadet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::CADET, 1)?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(deal_four, "This creature deals 4 damage to target creature or planeswalker.")
        .targets(&[targets::CREATURE_OR_PLANESWALKER]),
    mode(create_cadet, "Create a 2/2 colorless Wizard Soldier creature token named Cadet."),
    mode(|g, item| {
        g.draw(item.controller, 1);
        Ok(())
    }, "Draw a card."),
])
.choose(2, 2);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(cantrip_mechanics::triggers::THIS_ENTERS, &MODAL, "When this creature enters, choose two — This creature deals 4 damage to target creature or planeswalker; create a Cadet token; or draw a card.")],
    ..oracle::CRAFTWORK_CRUSHER
};
