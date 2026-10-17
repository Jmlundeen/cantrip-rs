use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::PROWESS};
use cantrip_oracle as oracle;

/// Improvised Act, the prepare spell (CR 722): "You may discard a card. If you do, draw a card." Not a card, only ever a
/// copy cast from exile.
pub static IMPROVISED_ACT: CardDef = CardDef {
    name: "Improvised Act",
    cost: mana("{R}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], |g, item| effects::may_discard_to_draw(g, item, 1)),
    text: "You may discard a card. If you do, draw a card.",
    ..CardDef::BLANK
};

fn enters_prepared(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.prepared = true;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[PROWESS],
    enters: Some(enters_prepared),
    prepare: Some(&IMPROVISED_ACT),
    ..oracle::POMPOUS_BATTLEMAGE
};

pub static EXTRA: &[&CardDef] = &[&IMPROVISED_ACT];
