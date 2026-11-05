use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Destroy all creatures and enchantments. Draw a card for each permanent destroyed this way" (CR 701.8a, 101.4, 608.2c): every
/// creature and every enchantment goes at once (an enchantment creature once), and the draw counts what was actually destroyed,
/// not an indestructible permanent nor one a regeneration shield saved.
fn destroy_then_draw_for_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) || g.chars(id).is(CardType::Enchantment))
        .collect();
    let destroyed = g.destroy_all(&doomed).len();
    effects::draw(g, item, destroyed as u32)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_then_draw_for_each),
    ..oracle::DEATH_BEGETS_LIFE
};
