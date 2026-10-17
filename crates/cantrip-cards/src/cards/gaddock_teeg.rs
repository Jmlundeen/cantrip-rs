use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Noncreature spells with mana value 4 or greater can't be cast." and "Noncreature spells with {X} in their mana costs
/// can't be cast." (CR 101.2, 601.3): for every player, judged on the spell as it would be cast - one half of a split card,
/// an Adventure, a back face (CR 601.3e, 709.3a) - and again on the stack once its proposal is complete (CR 601.6). Mana
/// value is the mana cost's alone, whatever is paid instead (CR 202.3, 118.9c); {X} is in the mana cost whatever X is.
fn allows(q: &dyn Query, _: &Asker, _: PlayerId, spell: ObjectId) -> bool {
    q.chars(spell).is(CardType::Creature) || (q.mana_value(spell) < 4 && q.mana_cost(spell).x == 0)
}

pub static CARD: CardDef = CardDef { spell_rules: &[allows], ..oracle::GADDOCK_TEEG };
