use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, bestow};
use cantrip_oracle as oracle;

fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

fn flying_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying).with(Keyword::Haste);
}

const STATICS: &[StaticDef] = &[
    StaticDef {
        text: "Enchanted creature gets +2/+2.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: plus_two_plus_two,
        ..StaticDef::BLANK
    },
    StaticDef {
        text: "Enchanted creature has flying and haste.",
        layer: Layer::Ability,
        affects: attached_object,
        apply: flying_and_haste,
        ..StaticDef::BLANK
    },
];

/// Bestow—{R}, Collect evidence 6 (CR 702.103, 701.59): while it is bestowed it is an Aura enchantment with "enchant
/// creature", not a creature, whose cost is the bestow cost. The evidence is an additional cost of casting it that way.
pub static BESTOWED: CardDef = bestow::bestowed(CardDef {
        cost: mana(0, 0, 0, 0, 1, 0),
        keywords: Keywords::of(&[]),
        additional_cost: Some(Cost { collect_evidence: 6, ..Cost::NONE }),
        statics: STATICS,
        ..oracle::DETECTIVES_PHOENIX
});

/// "You may cast this card from your graveyard using its bestow ability."
pub static CARD: CardDef = CardDef {
    statics: STATICS,
    bestow: Some(&BESTOWED),
    bestow_from_graveyard: true,
    ..oracle::DETECTIVES_PHOENIX
};

pub static EXTRA: &[&CardDef] = &[&BESTOWED];
