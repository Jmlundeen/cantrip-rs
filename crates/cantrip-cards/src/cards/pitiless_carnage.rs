use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::plot::plot;
use cantrip_oracle as oracle;

/// Sacrifice any number of permanents you control, then draw that many cards. The permanents are chosen as the spell
/// resolves (CR 701.21a, not targeted), including none, and sacrificed together; the draw is as many as were actually
/// sacrificed (CR 608.2c), so choosing none draws none.
fn sacrifice_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let yours = g.permanents_of(you);
    let chosen = choose::cards(g, you, item.object, Purpose::Sacrifice, yours.clone(), u8::MAX, true)?;
    let chosen: Vec<ObjectId> = yours.into_iter().filter(|c| chosen.contains(c)).collect();
    if chosen.is_empty() {
        return Ok(());
    }
    let sacrificed = g.sacrifice(&chosen).len();
    if sacrificed > 0 {
        g.draw(you, sacrificed as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect: sacrifice_and_draw }),
    abilities: &[plot(
        mana(1, 0, 0, 2, 0, 0),
        "Plot {1}{B}{B} (You may pay {1}{B}{B} and exile this card from your hand. Cast it as a sorcery on a later turn without paying its mana cost. Plot only as a sorcery.)",
    )],
    ..oracle::PITILESS_CARNAGE
};
