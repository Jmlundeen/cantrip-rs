use cantrip_api::*;
use cantrip_mechanics::damage_target;
use cantrip_mechanics::{attach::attached_object, targets};
use cantrip_oracle as oracle;

/// "This creature deals 1 damage to target player or planeswalker. You draw a card." The creature is the source, and
/// "you" is whoever controls it.
fn ping_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    delayed: &[ability(TAP, ping_and_draw, "{T}: This creature deals 1 damage to target player or planeswalker. You draw a card.")
        .targets(&[targets::PLAYER_OR_PLANESWALKER])],
    statics: &[static_ability(Layer::Ability, attached_object, cantrip_mechanics::statics::grant_delayed_ability, "Enchanted creature has \"{T}: This creature deals 1 damage to target player or planeswalker. You draw a card.\"")],
    ..oracle::QUICKSILVER_DAGGER
};
