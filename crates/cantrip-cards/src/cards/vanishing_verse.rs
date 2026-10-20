use cantrip_api::*;
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

/// "Target monocolored permanent" (CR 115.1): a permanent that is exactly one color. Colorless permanents are not
/// monocolored (CR 105.2a, 105.2c; ruling 2024-04-12), nor are those of two or more colors (CR 105.2b).
fn monocolored_permanent(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.chars(id).colors.0.count_ones() == 1)
}

/// "Exile target monocolored permanent." (CR 701.13a)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(monocolored_permanent)], effect),
    ..oracle::VANISHING_VERSE
};
