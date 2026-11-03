use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "It deals 1 damage to you": dealt by the land, off the status it last had if it has left.
fn damage_you(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, Target::Player(item.controller), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_BECOMES_TAPPED, damage_you, "Whenever this land becomes tapped, it deals 1 damage to you."),
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
    ],
    ..oracle::CITY_OF_BRASS
};
