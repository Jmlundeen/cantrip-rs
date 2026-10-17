use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

/// "Activate only if this land entered this turn or if you control a basic land" (CR 602.5b): checked as the ability is
/// activated. Entering is asked of the event history (CR 400.7: a land that left and came back is a new object that
/// entered this turn); a basic land is a land with the basic supertype (CR 205.4a), read through the layers. The land
/// itself is not basic, so it never counts itself.
fn entered_or_controls_basic(q: &dyn Query, asker: &Asker) -> bool {
    history::entered_this_turn(q.history(), asker.source)
        || q.permanents_of(asker.controller).into_iter().any(|id| {
            let chars = q.chars(id);
            chars.is(CardType::Land) && chars.supertypes.basic
        })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_one_of(&[Color::White, Color::Blue]), "{T}: Add {W} or {U}. Activate only if this land entered this turn or if you control a basic land.")
            .condition(entered_or_controls_basic),
    ],
    ..oracle::GLEAMING_BASTION
};
