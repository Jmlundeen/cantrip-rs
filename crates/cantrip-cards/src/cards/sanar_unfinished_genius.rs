use cantrip_api::*;
use cantrip_mechanics::library::{Find, instant_or_sorcery_card, search_to_hand};
use cantrip_mechanics::{prepare, tokens};
use cantrip_oracle as oracle;

/// Wild Idea: "Search your library for an instant or sorcery card, reveal it, put it into your hand, then shuffle." (CR 701.23): the
/// player may find nothing (CR 701.23b) and the library is shuffled either way.
fn wild_idea(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), instant_or_sorcery_card)?;
    Ok(())
}

/// Sanar's prepare spell (CR 722): not a card, only ever a copy.
pub static WILD_IDEA: CardDef = CardDef {
    name: "Wild Idea",
    cost: mana("{3}{U}{R}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], wild_idea),
    text: "Search your library for an instant or sorcery card, reveal it, put it into your hand, then shuffle.",
    ..CardDef::BLANK
};

/// "Activate only if you've cast an instant or sorcery spell this turn" (CR 602.5b): asked as the ability is activated. A copy of
/// a prepare spell that was cast counts like any spell cast (CR 722.3c, 601.2i); a copy of a spell put onto the stack is not cast.
fn cast_an_instant_or_sorcery_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.history().casts_by(asker.controller).any(|c| c.is(CardType::Instant) || c.is(CardType::Sorcery))
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, tokens::create_a_treasure, "{T}: Create a Treasure token. Activate only if you've cast an instant or sorcery spell this turn.")
        .condition(cast_an_instant_or_sorcery_this_turn)],
    enters: Some(prepare::enters_prepared),
    prepare: Some(&WILD_IDEA),
    ..oracle::SANAR_UNFINISHED_GENIUS
};

pub static EXTRA: &[&CardDef] = &[&WILD_IDEA];
