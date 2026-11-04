use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Lizard Formula - When Lizard, Connors's Curse enters, up to one other target creature loses all abilities and becomes a
/// green Lizard creature with base power and toughness 4/4." For good: the effect does not end at end of turn (the ruling).
fn lizard_formula(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::loses_all_abilities_and_becomes_a_creature_for_good(g, item, 0, Color::Green, Subtype::Lizard, 4, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        lizard_formula,
        "Lizard Formula \u{2014} When Lizard, Connors's Curse enters, up to one other target creature loses all abilities and becomes a green Lizard creature with base power and toughness 4/4.",
    )
    .targets(&[up_to_one(targets::other_creature_filter)])],
    ..oracle::LIZARD_CONNORSS_CURSE
};
