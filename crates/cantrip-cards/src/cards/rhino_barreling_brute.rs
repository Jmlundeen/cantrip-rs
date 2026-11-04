use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ATTACKS};

fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}
use cantrip_oracle as oracle;

/// "If you've cast a spell with mana value 4 or greater this turn" (CR 603.4, 202.3): an intervening "if", asked as the ability
/// would trigger and again as it resolves. Each spell the controller cast this turn counts by the mana value it had as it became
/// cast (X counted, CR 202.3e), whatever has become of it since - Rhino's own, cast this turn, included.
fn cast_a_spell_with_mana_value_4_or_greater(q: &dyn Query, asker: &Asker) -> bool {
    q.history().casts_by(asker.controller).any(|cast| cast.value(q) >= 4)
}

/// Vigilance, trample and haste are core keywords.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ATTACKS, draw_a_card, "Whenever Rhino attacks, if you've cast a spell with mana value 4 or greater this turn, draw a card.")
        .condition(cast_a_spell_with_mana_value_4_or_greater)],
    ..oracle::RHINO_BARRELING_BRUTE
};
