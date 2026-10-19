use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Noncreature spells you cast have improvise." (CR 702.126a, 601.2f-h): a spell that `caster`, the controller of Ironheart,
/// casts and that is not a creature spell, judged on the spell's characteristics as it is cast. The spell has improvise only
/// while it is cast and only while Ironheart is on the battlefield with its abilities (it is a static ability, CR 113.6); a
/// creature spell, and any spell an opponent casts, gets nothing.
fn noncreature_improvise(q: &dyn Query, source: ObjectId, caster: PlayerId, spell: ObjectId) -> PayWith {
    if q.controller(source) == caster && !q.chars(spell).is(CardType::Creature) {
        PayWith { improvise: true, ..PayWith::NONE }
    } else {
        PayWith::NONE
    }
}

pub static CARD: CardDef = CardDef { grants_pay_with: &[noncreature_improvise], ..oracle::IRONHEART_CLEVER_CHAMPION };
