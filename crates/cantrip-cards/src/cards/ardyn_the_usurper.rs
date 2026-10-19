use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Exile up to one target creature card from a graveyard. If you exiled a card this way, create a token that's a copy of
/// that card, except it's a 5/5 black Demon." (CR 707.2, 707.9b, 111.1): the copy is of the card as it was in the graveyard,
/// its printed values and nothing of the object it was before (the rulings), and the exception states what the token is: a
/// Demon creature instead of its other card types and creature types (CR 205.1a), black instead of its colors, with base
/// power and toughness 5/5, keeping its name, mana cost, supertypes (a legendary card gives a legendary token) and abilities.
/// "Black" and "Demon" are words of Ardyn's text (CR 612). If the exile did not happen (the target was illegal, or a
/// replacement effect sent the card elsewhere), no token is made.
fn exile_and_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let words = g.words(item.object);
    let values = g
        .copiable_values(card)
        .typed(TypeSet::of(&[CardType::Creature]), SubtypeSet::of(&[words.subtype(Subtype::Demon)]))
        .colored(ColorSet::of(words.color(Color::Black)))
        .sized(5, 5);
    let exiled = g.exile_all(&[card]);
    g.create_token_copies_of(values, item.controller, exiled.len() as u32)?;
    Ok(())
}

/// "Demons you control" (CR 613.1f, layer 6): creatures under the source's controller's control with the Demon type, as the
/// earlier layers made them (a token that is a Demon counts), the type read through the source's text (CR 612).
fn demons_you_control(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    view.state().obj(affected).controller == view.state().obj(source).controller
        && chars.is(CardType::Creature)
        && chars.has_subtype(view.words(source).subtype(Subtype::Demon))
}

/// "... have menace, lifelink, and haste" (CR 702.110, 702.15, 702.10).
fn have_menace_lifelink_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Menace).with(Keyword::Lifelink).with(Keyword::Haste);
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::BEGINNING_OF_YOUR_COMBAT,
        exile_and_copy,
        "Starscourge \u{2014} At the beginning of combat on your turn, exile up to one target creature card from a graveyard. If you exiled a card this way, create a token that's a copy of that card, except it's a 5/5 black Demon.",
    )
    .targets(&[up_to_one(targets::CREATURE_CARD_IN_A_GRAVEYARD.filter)])],
    statics: &[static_ability(
        Layer::Ability,
        demons_you_control,
        have_menace_lifelink_and_haste,
        "Demons you control have menace, lifelink, and haste.",
    )],
    ..oracle::ARDYN_THE_USURPER
};
