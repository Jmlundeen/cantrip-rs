use cantrip_api::*;
use cantrip_mechanics::{effects, lands, triggers};
use cantrip_oracle as oracle;

/// "Spider spells and noncreature spells" (CR 601.3, 205.3): the card on top as the spell it would be, a Spider (its subtype
/// read through the words of this card's text, CR 612) or not a creature.
fn spider_or_noncreature(q: &dyn Query, asker: &Asker, card: ObjectId) -> bool {
    let c = q.chars(card);
    c.has_subtype(lands::word(q, asker.source, lands::sub(Subtype::Spider))) || !c.is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    // "You may look at the top card of your library any time" (CR 401.2), and "you may cast Spider spells and noncreature
    // spells from the top of your library" (CR 601.3).
    play_rules: &[PlayRule::LookAtLibraryTop, PlayRule::LibraryTopSpells(spider_or_noncreature, None)],
    abilities: &[triggered(triggers::YOU_ATTACK, effects::may_mill_a_card, "Whenever you attack, you may mill a card.")],
    ..oracle::MADAME_WEB_CLAIRVOYANT
};
