use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Counter target spell, activated ability, or triggered ability." (CR 115.1, 701.6).
fn counter_spell_or_ability(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(spell) = item.target(0) {
        g.counter(spell);
    } else if let Target::Ability(_) = item.target(0) {
        let _ = cantrip_mechanics::counter_target_ability(g, item, 0);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL_OR_ABILITY], counter_spell_or_ability),
    ..oracle::DISALLOW
};
