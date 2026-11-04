use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target opponent gains control of another target permanent you control. If they do, you draw a card." (CR 108.4, 608.2c):
/// no card is drawn if either target has become illegal, nor if the permanent did not change hands (CR 608.2b, 400.7, 800.4b).
fn trade(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (Some(opponent), Some(permanent)) = (targets::player(g, item, 0), targets::object(g, item, 1)) else { return Ok(()) };
    let they_do = g.gain_control(item.object, permanent, opponent, Duration::Indefinite);
    g.end_sentence();
    if they_do { effects::draw(g, item, 1)? }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}, {T}"),
        trade,
        "{2}, {T}: Target opponent gains control of another target permanent you control. If they do, you draw a card.",
    )
    .targets(&[
        targets::OPPONENT.labelled("Opponent who gains control of the permanent"),
        targets::ANOTHER_PERMANENT_YOU_CONTROL.labelled("Permanent you control for that opponent to gain control of"),
    ])],
    ..oracle::STILTZKIN_MOOGLE_MERCHANT
};
