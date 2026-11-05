use cantrip_api::*;
use cantrip_mechanics::targets::{is_permanent_card, permanent_cards_in_graveyard};
use cantrip_mechanics::triggers::{THIS_ENTERS, YOUR_END_STEP};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// The end step ability's intervening "if" (CR 603.4): six or more permanent cards (CR 110.4a) in its controller's
/// graveyard, checked as the end step begins and again as the ability resolves.
fn six_permanent_cards_in_your_graveyard(q: &dyn Query, asker: &Asker) -> bool {
    permanent_cards_in_graveyard(q, asker.controller) >= 6
}

/// "Neo Exdeath's power is equal to the number of permanent cards in your graveyard" (CR 604.3, layer 7a): its controller's
/// graveyard, each card judged as it is there.
fn power_is_permanent_cards_in_your_graveyard(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let you = q.state().obj(source).controller;
    let n = q.state().zone(Zone::graveyard(you)).iter().filter(|&&id| is_permanent_card(&q.chars(id))).count();
    chars.power = Some(n as i32);
}

pub static BACK: CardDef = CardDef {
    statics: &[
        characteristic_defining(
            Layer::PtDefine,
            power_is_permanent_cards_in_your_graveyard,
            "Neo Exdeath's power is equal to the number of permanent cards in your graveyard.",
        )
        .lost_pt(0, 3),
    ],
    ..oracle::NEO_EXDEATH_DIMENSIONS_END
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 3), "When Exdeath enters, you gain 3 life."),
        triggered(
            YOUR_END_STEP,
            effects::transform_self,
            "At the beginning of your end step, if there are six or more permanent cards in your graveyard, transform Exdeath.",
        )
        .condition(six_permanent_cards_in_your_graveyard),
    ],
    back: Some(&BACK),
    ..oracle::EXDEATH_VOID_WARLOCK
};
