use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// Return target card from your graveyard to your hand.
fn regrowth(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

/// Emeritus of Abundance's prepare spell (CR 722): not a card here, only ever a copy.
pub static REGROWTH: CardDef = CardDef {
    name: "Regrowth",
    cost: mana("{1}{G}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[targets::CARD_IN_YOUR_GRAVEYARD], regrowth),
    text: "Return target card from your graveyard to your hand.",
    ..CardDef::BLANK
};

/// "This creature enters prepared" (CR 614.1d, 722.3a).
fn enters_prepared(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.prepared = true;
    Ok(())
}

/// "If you control eight or more lands" (CR 603.4): checked as it triggers and again as it resolves.
fn eight_lands(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().filter(|&id| q.chars(id).is(CardType::Land)).count() >= 8
}

fn become_prepared(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.become_prepared(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, become_prepared, "Whenever this creature attacks, if you control eight or more lands, this creature becomes prepared.")
        .condition(eight_lands)],
    enters: Some(enters_prepared),
    prepare: Some(&REGROWTH),
    ..oracle::EMERITUS_OF_ABUNDANCE
};

pub static EXTRA: &[&CardDef] = &[&REGROWTH];
