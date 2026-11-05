use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::mana::enters_tapped;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// "This ability costs {1} less to activate for each other Town you control" (CR 601.2f, 118.7): the Towns the activating player
/// controls as the layers have made them, this land itself not among them. Generic mana only.
fn cheaper_per_other_town(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    let town = lands::word(q, asker.source, lands::sub(Subtype::Town));
    let others = q.permanents_of(asker.controller).into_iter().filter(|&id| id != asker.source && q.chars(id).has_subtype(town)).count();
    CostDelta::less(others as u32)
}

/// "Whenever Balamb Garden attacks, draw a card" (CR 508.1m, 121.1).
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS, draw_a_card, "Whenever Balamb Garden attacks, draw a card."),
        crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::BALAMB_GARDEN_AIRBORNE
};

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Green, Color::Blue]), "{T}: Add {G} or {U}."),
        ability(
            cost("{5}{G}{U}").and(TAP),
            effects::transform_self,
            "{5}{G}{U}, {T}: Transform this land. This ability costs {1} less to activate for each other Town you control.",
        )
        .cost_change(cheaper_per_other_town),
    ],
    back: Some(&BACK),
    ..oracle::BALAMB_GARDEN_SEED_ACADEMY
};
