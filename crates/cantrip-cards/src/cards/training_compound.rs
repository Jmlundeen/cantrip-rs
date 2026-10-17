use cantrip_api::*;
use cantrip_mechanics::history::entered_this_turn;
use cantrip_oracle as oracle;

/// "Activate only if this land entered this turn or if you control a basic land" (CR 602.5b), checked as the ability is activated:
/// this land is the object that entered this turn (CR 400.7), or a land with the basic supertype (CR 205.4a) is among the permanents
/// its controller controls through the layers. This land is not basic itself, so it never counts as the basic land.
fn entered_this_turn_or_you_control_a_basic_land(q: &dyn Query, asker: &Asker) -> bool {
    entered_this_turn(q.history(), asker.source)
        || q.permanents_of(asker.controller).into_iter().any(|id| {
            let c = q.chars(id);
            c.is(CardType::Land) && c.supertypes.basic
        })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(
            TAP,
            add_one_of(&[Color::Red, Color::Green]),
            "{T}: Add {R} or {G}. Activate only if this land entered this turn or if you control a basic land.",
        )
        .condition(entered_this_turn_or_you_control_a_basic_land),
    ],
    ..oracle::TRAINING_COMPOUND
};
