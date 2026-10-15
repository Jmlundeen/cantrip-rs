use cantrip_api::*;
use cantrip_oracle as oracle;

/// Pay 7 life: Draw seven cards. The life is the whole cost (CR 119.4).
fn draw_seven(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 7);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Pay 7 life: Draw seven cards.",
        cost: Cost { life: 7, ..Cost::NONE },
        effect: draw_seven,
        ..ActivatedDef::BLANK
    })],
    ..oracle::GRISELBRAND
};
