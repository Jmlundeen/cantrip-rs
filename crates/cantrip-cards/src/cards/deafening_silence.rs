use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Each player can't cast more than one noncreature spell each turn." (CR 101.2, 601.3.) Judged on the spell about to be
/// cast, for its caster: it may not be cast when that player has cast a noncreature spell this turn, whether or not this
/// enchantment was out then, and a countered spell still counts (rulings 2019-10-04). The cast is recorded only once it is
/// made (CR 601.2i), so the history holds just the spells cast before this one, also when asked again once the proposal is
/// complete (CR 601.6).
fn allows(q: &dyn Query, _: &Asker, player: PlayerId, spell: ObjectId) -> bool {
    q.chars(spell).is(CardType::Creature) || cantrip_mechanics::history::first_noncreature_spell_this_turn(q, spell, player)
}

pub static CARD: CardDef = CardDef { spell_rules: &[allows], ..oracle::DEAFENING_SILENCE };
