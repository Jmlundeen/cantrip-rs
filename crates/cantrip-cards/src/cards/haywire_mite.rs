use cantrip_api::*;
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Haywire Mite dies, you gain 2 life.",
            listens: EventMask::ZONE_CHANGE,
            matches: cantrip_mechanics::triggers::this_dies,
            effect: cantrip_mechanics::effects::gain_life::<2>,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{G}, Sacrifice Haywire Mite: Exile target noncreature artifact or noncreature enchantment.",
            cost: Cost { mana: mana(0, 0, 0, 0, 0, 1), sacrifice_self: true, ..Cost::NONE },
            targets: &[targets::NONCREATURE_ARTIFACT_OR_ENCHANTMENT],
            effect: exile,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::HAYWIRE_MITE
};
