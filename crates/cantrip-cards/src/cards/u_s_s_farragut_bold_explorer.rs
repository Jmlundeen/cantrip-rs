use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{delayed, effects, hand, station, statics};
use cantrip_oracle as oracle;

const SEVEN_PLUS: &str = "7+ | Flying";

const LANDFALL_TEXT: &str = "Landfall \u{2014} Whenever a land you control enters, you may put a creature card from your hand onto the battlefield. That creature gains haste. At the beginning of the next end step, put it on the bottom of your library. When you do, draw a card.";

/// "You may put a creature card from your hand onto the battlefield. That creature gains haste." (CR 608.2d, 611.2a): the card
/// is chosen as the trigger resolves, and nobody is asked with no creature card in hand. Haste has no stated duration, so it
/// lasts as long as it is that object; the delayed ability at the next end step is set up about it (CR 603.7c).
fn may_put_then_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(entered) = hand::may_put_onto_battlefield(g, item.controller, item.object, |c| c.is(CardType::Creature))? else {
        return Ok(());
    };
    if !g.is_on_battlefield(entered) { return Ok(()); }
    effects::gain_haste(g, item.object, entered);
    delayed::at_next_end_step(g, item, 0, Target::Object(entered));
    Ok(())
}

/// "At the beginning of the next end step, put it on the bottom of your library. When you do, draw a card." (CR 603.7c, 400.3,
/// 603.12): the delayed ability's subject, if it is still the object it was when the ability was set up, goes to the bottom of
/// its owner's library; the reflexive ability triggers only if it really was put there.
fn put_subject_on_bottom_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(it) = item.subject else { return Ok(()) };
    if !is_live(g, it) { return Ok(()); }
    // Phased out, or sent elsewhere (a commander to the command zone): nothing was put there, so nothing triggers.
    if g.put_into_library(&[it], LibraryPlace::Bottom).is_empty() { return Ok(()); }
    g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(item.ability.card, 1), Target::None);
    Ok(())
}

/// "When you do, draw a card." (CR 603.12, 121.1).
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(LANDFALL, may_put_then_haste, LANDFALL_TEXT),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 7+.)"),
    ],
    statics: &[
        // "{7+}[abilities][P/T]" (CR 721.2a-b): while it has seven or more charge counters it is an artifact creature with base
        // power and toughness 4/4 in addition to its other types, and has the striation's static ability.
        static_ability(Layer::Type, station::charged::<7>, station::is_a_creature, SEVEN_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<4, 4>, SEVEN_PLUS).continues(),
    ],
    delayed: &[
        delayed_trigger(put_subject_on_bottom_then_draw, "At the beginning of the next end step, put it on the bottom of your library. When you do, draw a card."),
        delayed_trigger(draw_a_card, "When you do, draw a card."),
    ],
    ..oracle::U_S_S_FARRAGUT_BOLD_EXPLORER
};
