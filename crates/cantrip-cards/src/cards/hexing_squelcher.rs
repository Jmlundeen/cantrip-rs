use cantrip_api::*;
use cantrip_mechanics::triggers::ward_life;
use cantrip_oracle as oracle;

const WARD: AbilityDef = ward_life::<2>("Ward\u{2014}Pay 2 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 2 life.)");

fn other_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    affected != source && so_far.is(CardType::Creature) && q.state().obj(affected).controller == q.state().obj(source).controller
}

pub static CARD: CardDef = CardDef {
    abilities: &[WARD],
    delayed: &[WARD],
    statics: &[
        static_ability(Layer::Ability, cantrip_mechanics::statics::your_spells_of_types::<{ cantrip_mechanics::statics::ANY_SPELL }>, cantrip_mechanics::statics::cant_be_countered, "Spells you control can't be countered.")
            .everywhere(),
        static_ability(Layer::Ability, other_creatures_you_control, cantrip_mechanics::statics::grant_delayed_ability, "Other creatures you control have \"Ward\u{2014}Pay 2 life.\""),
    ],
    ..oracle::HEXING_SQUELCHER
};
