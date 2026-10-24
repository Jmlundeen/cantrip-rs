use cantrip_api::*;
use cantrip_mechanics::{statics, tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast a noncreature spell, create a 1/1 colorless Sculpture Treasure artifact creature token"
/// (CR 603.2, 701.7a).
fn make_a_sculpture(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &SCULPTURE_TREASURE, 1)?;
    Ok(())
}

/// The Sculpture Treasure has the mana ability printed by Vraska (CR 605.1a).
static SCULPTURE_TREASURE: CardDef = CardDef {
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[mana_ability(TAP.and(SACRIFICE_THIS), add_any_color(1), "{T}, Sacrifice this token: Add one mana of any color.")],
    text: "{T}, Sacrifice this token: Add one mana of any color.",
    ..token!(Sculpture Treasure)
};

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, statics::your_artifact_creatures, statics::have_vigilance, "Artifact creatures you control have vigilance.")],
    abilities: &[triggered(triggers::NONCREATURE_SPELL_YOU_CAST, make_a_sculpture, "Whenever you cast a noncreature spell, create a 1/1 colorless Sculpture Treasure artifact creature token with \"{T}, Sacrifice this token: Add one mana of any color.\"")],
    ..oracle::VRASKA_SOUL_OF_STONE
};

pub static EXTRA: &[&CardDef] = &[&SCULPTURE_TREASURE];
