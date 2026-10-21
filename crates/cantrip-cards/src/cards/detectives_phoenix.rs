use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, bestow};
use cantrip_oracle as oracle;

fn flying_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying).with(Keyword::Haste);
}

const STATICS: &[StaticDef] = &[
    static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<2, 2>, "Enchanted creature gets +2/+2."),
    static_ability(Layer::Ability, attached_object, flying_and_haste, "Enchanted creature has flying and haste."),
];

/// Bestow—{R}, Collect evidence 6 (CR 702.103, 701.59): while it is bestowed it is an Aura enchantment with "enchant
/// creature", not a creature, with the card's mana cost (CR 702.103b); the bestow cost is the card's `bestow_cost`. The
/// evidence is an additional cost of casting it that way.
pub static BESTOWED: CardDef = bestow::bestowed(CardDef {
        keywords: Keywords::of(&[]),
        additional_cost: Some(Cost { collect_evidence: 6, ..Cost::NONE }),
        statics: STATICS,
        ..oracle::DETECTIVES_PHOENIX
});

/// "You may cast this card from your graveyard using its bestow ability."
pub static CARD: CardDef = CardDef {
    statics: STATICS,
    bestow: Some(&BESTOWED),
    bestow_cost: mana("{R}"),
    bestow_from_graveyard: true,
    ..oracle::DETECTIVES_PHOENIX
};

pub static EXTRA: &[&CardDef] = &[&BESTOWED];
