use cantrip_api::*;
use cantrip_mechanics::{station, statics};
use cantrip_oracle as oracle;

/// The 10+ striation, "Flying, vigilance" with the 6/6 box (CR 721.2a-b).
const TEN_PLUS: &str = "10+ | Flying, vigilance (6/6)";

/// "Klingons and Warriors you control have vigilance." (CR 613.1f, layer 6, 702.20a): every creature under the artifact's
/// controller that is either creature type as the earlier layers made it - the types read through the text's own words
/// (CR 612.2) - and not another player's creature.
fn klingons_and_warriors_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let words = q.words(source);
    let of_either_type = so_far.has_subtype(words.subtype(Subtype::Klingon)) || so_far.has_subtype(words.subtype(Subtype::Warrior));
    let s = q.state();
    so_far.is(CardType::Creature) && of_either_type && s.obj(affected).controller == s.obj(source).controller
}

/// "Flying, vigilance" of the 10+ striation (CR 702.9a, 702.20a), added in layer 6 as one part of one static ability.
fn have_flying_and_vigilance(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(q, source, chars);
    statics::have_vigilance(q, source, chars);
}

pub static CARD: CardDef = CardDef {
    abilities: &[station::ability(
        "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 10+.)",
    )],
    statics: &[
        static_ability(Layer::Ability, klingons_and_warriors_you_control, statics::have_vigilance, "Klingons and Warriors you control have vigilance."),
        static_ability(Layer::Type, station::charged::<10>, station::is_a_creature, TEN_PLUS),
        static_ability(Layer::Ability, station::charged::<10>, have_flying_and_vigilance, TEN_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<10>, statics::base_pt::<6, 6>, TEN_PLUS).continues(),
    ],
    ..oracle::I_K_S_NEGHVAR_IMPERIAL_FLAGSHIP
};
