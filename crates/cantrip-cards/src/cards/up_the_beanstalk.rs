use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "When this enchantment enters and whenever you cast a spell with mana value 5 or greater" (CR 603.2, 603.6a): one ability with
/// two events.
fn enters_or_you_cast_a_spell_with_mana_value_5_or_greater(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    triggers::this_enters(q, me.source, event).or_else(|| triggers::you_cast_a_spell_with_mana_value_at_least::<5>(q, me, event))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask(EventMask::ZONE_CHANGE.0 | EventMask::SPELL_CAST.0), enters_or_you_cast_a_spell_with_mana_value_5_or_greater),
        |g, item| effects::draw(g, item, 1),
        "When this enchantment enters and whenever you cast a spell with mana value 5 or greater, draw a card.",
    )],
    ..oracle::UP_THE_BEANSTALK
};
