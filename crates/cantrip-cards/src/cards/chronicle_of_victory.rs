use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

fn first_strike_and_trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::FirstStrike).with(Keyword::Trample);
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::chosen_type::enters_choosing_a_creature_type),
    abilities: &[triggered(when_asked(EventMask::SPELL_CAST, cantrip_mechanics::chosen_type::you_cast_chosen_type), |g, item| effects::draw(g, item, 1), "Whenever you cast a spell of the chosen type, draw a card.")],
    statics: &[
        static_ability(Layer::PtModify, cantrip_mechanics::chosen_type::creatures_you_control_of_chosen_type, cantrip_mechanics::statics::plus_pt::<2, 2>, "Creatures you control of the chosen type get +2/+2."),
        static_ability(Layer::Ability, cantrip_mechanics::chosen_type::creatures_you_control_of_chosen_type, first_strike_and_trample, "Creatures you control of the chosen type have first strike and trample."),
    ],
    ..oracle::CHRONICLE_OF_VICTORY
};
