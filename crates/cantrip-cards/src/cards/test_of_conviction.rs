use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Begin Negotiations -- Target opponent draws a card. You draw two cards." The draws are two sentences (CR 608.2c): the
/// opponent is asked to draw for no one, then this spell's controller draws two.
fn begin_negotiations(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(opponent) = targets::player(g, item, 0) {
        g.draw(opponent, 1);
    }
    g.end_sentence();
    effects::draw(g, item, 2)
}

/// "Choose one --" (CR 700.2): one mode, chosen as the spell is cast, in printed order.
static MODAL: ModalDef = modes(&[
    mode(
        effects::plus_one_counter_on_each_creature_you_control,
        "Battle Stations \u{2014} Put a +1/+1 counter on each creature you control.",
    ),
    mode(
        begin_negotiations,
        "Begin Negotiations \u{2014} Target opponent draws a card. You draw two cards.",
    )
    .targets(&[targets::OPPONENT]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::TEST_OF_CONVICTION
};
