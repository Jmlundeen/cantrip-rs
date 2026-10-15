use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Other creatures you control".
fn other_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    affected != source && so_far.is(CardType::Creature) && s.obj(affected).controller == s.obj(source).controller
}

fn plus_one_plus_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
    chars.toughness = chars.toughness.map(|v| v + 1);
}

fn trample_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Haste);
}

pub static BACK: CardDef = CardDef {
    statics: &[
        StaticDef {
            text: "Other creatures you control get +1/+1.",
            layer: Layer::PtModify,
            affects: other_creatures_you_control,
            apply: plus_one_plus_one,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Other creatures you control have trample and haste.",
            layer: Layer::Ability,
            affects: other_creatures_you_control,
            apply: trample_and_haste,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::ULVENWALD_BEHEMOTH
};

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{5}{G}{G}: Transform this creature.",
        cost: Cost { mana: mana(5, 0, 0, 0, 0, 2), ..Cost::NONE },
        effect: effects::transform_self,
        ..ActivatedDef::BLANK
    })],
    back: Some(&BACK),
    ..oracle::ULVENWALD_ODDITY
};
