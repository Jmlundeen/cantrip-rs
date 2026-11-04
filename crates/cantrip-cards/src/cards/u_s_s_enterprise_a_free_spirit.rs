use cantrip_api::*;
use cantrip_mechanics::triggers::ward;
use cantrip_mechanics::{station, statics, tokens, triggers};
use cantrip_oracle as oracle;

/// "When U.S.S. Enterprise-A enters, create three 1/1 red Officer creature tokens."
fn three_officers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OFFICER, 3)?;
    Ok(())
}

/// The 6+ striation, "Flying, lifelink" with the 5/5 power/toughness box (CR 721.2b): while it has six or more charge
/// counters it has flying and lifelink and is a creature with base power and toughness 5/5 in addition to its other types.
/// One static ability, applied in each layer it touches (CR 613.6).
const SIX_PLUS: &str = "6+ | Flying, lifelink (5/5)";

/// The striation's two keywords (CR 702.9, 702.15), layer 6.
fn have_flying_and_lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying).with(Keyword::Lifelink);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)"),
        triggered(
            triggers::THIS_ENTERS,
            three_officers,
            "When U.S.S. Enterprise-A enters, create three 1/1 red Officer creature tokens.",
        ),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 6+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<6>, station::is_a_creature, SIX_PLUS),
        static_ability(Layer::Ability, station::charged::<6>, have_flying_and_lifelink, SIX_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<6>, statics::base_pt::<5, 5>, SIX_PLUS).continues(),
    ],
    ..oracle::U_S_S_ENTERPRISE_A_FREE_SPIRIT
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
