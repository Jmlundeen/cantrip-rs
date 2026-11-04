use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::statics::{base_pt, have_hexproof, itself_while_you_control_another};
use cantrip_oracle as oracle;

/// "As long as there are eight or more cards in your graveyard" (CR 604.1, 611.3a): only the source itself, and only while its
/// controller's graveyard holds eight or more cards, asked again whenever characteristics are computed. The opponent's graveyard
/// is another zone and does not count.
fn itself_with_eight_cards_in_your_graveyard(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = q.state();
    source == affected && state.zone(Zone::graveyard(state.obj(source).controller)).len() >= 8
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtSet,
            itself_with_eight_cards_in_your_graveyard,
            base_pt::<8, 8>,
            "As long as there are eight or more cards in your graveyard, Doc Ock has base power and toughness 8/8.",
        ),
        static_ability(
            Layer::Ability,
            itself_while_you_control_another::<{ lands::sub(Subtype::Villain) }>,
            have_hexproof,
            "As long as you control another Villain, Doc Ock has hexproof.",
        ),
    ],
    ..oracle::DOC_OCK_SINISTER_SCIENTIST
};
