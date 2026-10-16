use cantrip_api::*;
use cantrip_mechanics::triggers::CREATURE_SPELL_YOU_CAST;
use cantrip_oracle as oracle;

/// The spell creates its delayed triggered ability with a stated duration (CR 603.7b): "whenever you cast a creature
/// spell this turn", which may trigger any number of times. "You" is the controller of the spell as it resolved
/// (CR 603.7d).
fn watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::delayed::this_turn(g, item, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], watch),
    delayed: &[triggered(CREATURE_SPELL_YOU_CAST, |g, item| cantrip_mechanics::effects::draw(g, item, 1), "Whenever you cast a creature spell this turn, draw a card.")],
    ..oracle::GLIMPSE_OF_NATURE
};
