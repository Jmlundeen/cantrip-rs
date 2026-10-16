use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// The colorless 3/3 Robot Villain artifact creature token named Doombot (CR 111.4): its name is "Doombot", it has no
/// mana cost or color, and its subtypes are Robot and Villain.
pub static DOOMBOT: CardDef = CardDef {
    name: "Doombot",
    has_cost: false,
    colors: Some(ColorSet::NONE),
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Robot, Subtype::Villain]),
    power: Some(3),
    toughness: Some(3),
    ..CardDef::BLANK
};

/// "Create a 3/3 colorless Robot Villain artifact creature token named Doombot." The artifact was sacrificed as a cost.
fn create_doombot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &DOOMBOT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_any_color(1).only_for(ManaRestriction::ARTIFACT_SPELLS), "{T}: Add one mana of any color. Spend this mana only to cast an artifact spell."),
        ability(cost("{3}, {T}").and(sacrifice(1, targets::filters::artifact)), create_doombot, "{3}, {T}, Sacrifice an artifact: Create a 3/3 colorless Robot Villain artifact creature token named Doombot. Activate only as a sorcery.")
            .sorcery_speed(),
    ],
    ..oracle::CASTLE_DOOM
};

pub static EXTRA: &[&CardDef] = &[&DOOMBOT];
