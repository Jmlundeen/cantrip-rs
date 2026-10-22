use cantrip_api::*;
use cantrip_mechanics::triggers::NONCREATURE_SPELL_YOU_CAST;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Whenever you cast a noncreature spell, draw a card." The trigger resolves before the spell that caused it and
    // draws even if that spell is countered (rulings 2020-04-17, CR 603.2).
    abilities: &[triggered(
        NONCREATURE_SPELL_YOU_CAST,
        |g, item| cantrip_mechanics::effects::draw(g, item, 1),
        "Whenever you cast a noncreature spell, draw a card.",
    )],
    ..oracle::WHIRLWIND_OF_THOUGHT
};
