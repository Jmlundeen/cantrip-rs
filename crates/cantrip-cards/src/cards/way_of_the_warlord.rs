use cantrip_api::*;
use cantrip_mechanics::{damage_each_target, empower, statics, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When Way of the Warlord enters, empower Jace 5."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 5)
}

/// "[-4]: This planeswalker deals 2 damage to up to one target creature or planeswalker and 2 damage to target player." The
/// planeswalker that has the ability is the item's source; each target is dealt 2 only if still legal (CR 608.2b).
fn two_damage_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_each_target(g, item, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, empower_jace, "When Way of the Warlord enters, empower Jace 5.")],
    delayed: &[ability(
        loyalty(-4),
        two_damage_each,
        "\u{2212}4: This planeswalker deals 2 damage to up to one target creature or planeswalker and 2 damage to target player.",
    )
    .targets(&[
        targets::CREATURE_OR_PLANESWALKER.optional().labelled("Creature or planeswalker to be dealt 2 damage"),
        targets::PLAYER.labelled("Player to be dealt 2 damage"),
    ])],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_planeswalkers,
        statics::grant_delayed_ability,
        "Planeswalkers you control have \"\u{2212}4: This planeswalker deals 2 damage to up to one target creature or planeswalker and 2 damage to target player.\"",
    )],
    ..oracle::WAY_OF_THE_WARLORD
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
