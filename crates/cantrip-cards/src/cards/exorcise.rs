use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::exile_target;
use cantrip_oracle as oracle;

/// "Target artifact, enchantment, or creature with power 4 or greater": the power condition belongs to the creature
/// alone, so an artifact or enchantment of any size (an artifact creature too) is a legal target.
fn target_filter(q: &dyn Query, _: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    if !targets::permanent(q, id) {
        return false;
    }
    let c = q.chars(id);
    c.is(CardType::Artifact) || c.is(CardType::Enchantment) || (c.is(CardType::Creature) && c.power.is_some_and(|p| p >= 4))
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: target_filter, label: "" }], effect }),
    ..oracle::EXORCISE
};
