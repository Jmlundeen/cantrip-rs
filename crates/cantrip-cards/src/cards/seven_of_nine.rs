use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever you face a dilemma, draw a card." (CR 700.2a, 700.2b, 603.2): a dilemma is faced once as one or more modes of
/// a spell or ability are chosen, however many modes were chosen; the trigger fires once and its controller draws.
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_FACE_A_DILEMMA,
        draw_a_card,
        "Whenever you face a dilemma, draw a card.",
    )],
    ..oracle::SEVEN_OF_NINE
};
