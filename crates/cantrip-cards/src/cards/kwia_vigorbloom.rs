use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// The colorless artifact token named Lotus with "{T}, Sacrifice this token: Add three mana of any one color."
pub static LOTUS: CardDef = CardDef {
    name: "Lotus",
    has_cost: false,
    colors: Some(ColorSet::NONE),
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[mana_ability(
        TAP.and(SACRIFICE_THIS),
        add_any_color(3),
        "{T}, Sacrifice this token: Add three mana of any one color.",
    )],
    text: "{T}, Sacrifice this token: Add three mana of any one color.",
    ..CardDef::BLANK
};

fn make_lotus(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &LOTUS, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::ward::<2>(
            "Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)",
        ),
        triggered(
            triggers::YOU_GAIN_LIFE,
            make_lotus,
            "Whenever you gain life, create a colorless artifact token named Lotus with \"{T}, Sacrifice this token: Add three mana of any one color.\" This ability triggers only once each turn.",
        )
        .triggers_each_turn(1),
    ],
    ..oracle::KWIA_VIGORBLOOM
};

pub static EXTRA: &[&CardDef] = &[&LOTUS];
