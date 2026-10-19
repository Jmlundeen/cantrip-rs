use cantrip_api::*;
use cantrip_mechanics::statics::{have_haste, plus_pt};
use cantrip_oracle as oracle;

/// "As long as there are two or more instant and/or sorcery cards in your graveyard" (CR 604.1, 611.3a): only the source
/// itself, judged on the cards' characteristics as the layers made them, in its controller's graveyard.
fn with_two_instants_or_sorceries_in_graveyard(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = view.state();
    let you = state.obj(source).controller;
    source == affected
        && state
            .zone(Zone::graveyard(you))
            .iter()
            .filter(|&&card| view.chars(card).is(CardType::Instant) || view.chars(card).is(CardType::Sorcery))
            .count()
            >= 2
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            with_two_instants_or_sorceries_in_graveyard,
            plus_pt::<1, 0>,
            "As long as there are two or more instant and/or sorcery cards in your graveyard, this creature gets +1/+0.",
        ),
        static_ability(
            Layer::Ability,
            with_two_instants_or_sorceries_in_graveyard,
            have_haste,
            "As long as there are two or more instant and/or sorcery cards in your graveyard, this creature has haste.",
        ),
    ],
    ..oracle::GHITU_LAVARUNNER
};
