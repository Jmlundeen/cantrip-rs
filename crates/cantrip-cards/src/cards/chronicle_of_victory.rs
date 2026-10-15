use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

fn plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

fn first_strike_and_trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::FirstStrike).with(Keyword::Trample);
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::chosen_type::enters_choosing_a_creature_type),
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you cast a spell of the chosen type, draw a card.",
        listens: EventMask::SPELL_CAST,
        matches: cantrip_mechanics::chosen_type::you_cast_chosen_type,
        effect: effects::draw::<1>,
        ..TriggeredDef::BLANK
    })],
    statics: &[
        StaticDef {
            text: "Creatures you control of the chosen type get +2/+2.",
            layer: Layer::PtModify,
            affects: cantrip_mechanics::chosen_type::creatures_you_control_of_chosen_type,
            apply: plus_two,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Creatures you control of the chosen type have first strike and trample.",
            layer: Layer::Ability,
            affects: cantrip_mechanics::chosen_type::creatures_you_control_of_chosen_type,
            apply: first_strike_and_trample,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::CHRONICLE_OF_VICTORY
};
