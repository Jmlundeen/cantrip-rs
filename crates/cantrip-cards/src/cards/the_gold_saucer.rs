use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "{2}, {T}: Flip a coin. If you win the flip, create a Treasure token." (CR 705, 111.10a).
fn flip_coin_if_win_flip(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.flip_coin(item.controller) {
        tokens::create_treasure(g, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}").and(TAP), flip_coin_if_win_flip, "{2}, {T}: Flip a coin. If you win the flip, create a Treasure token."),
        ability(cost("{3}").and(TAP).and(sacrifice(2, targets::filters::artifact)), |g, item| effects::draw(g, item, 1), "{3}, {T}, Sacrifice two artifacts: Draw a card."),
    ],
    ..oracle::THE_GOLD_SAUCER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
