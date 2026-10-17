use cantrip_api::*;
use cantrip_mechanics::statics::all_creatures;
use cantrip_mechanics::triggers::{EACH_END_STEP, THIS_ENTERS};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Creatures lose all abilities." A static ability (CR 604.1) whose effect is in layer 6 (CR 613.1f), at Dress Down's
/// timestamp: what a creature gains after it keeps (the ruling: a land animated later keeps what its animation gives it),
/// everything it had before goes, mana and characteristic-defining abilities among them (a Tarmogoyf is 0/1, CR 208.2a). A
/// creature entering while it applies enters with none of its own "enters with" abilities (CR 614.12) and none of its
/// enters triggers is there to trigger (CR 603.6a); one dying with none has no dies trigger to look back to (CR 603.10a).
/// Made a creature itself (Opalescence), Dress Down loses its own abilities, its end-step sacrifice among them (the ruling),
/// and this effect goes on applying (CR 613.6).
const TEXT: &str = "Creatures lose all abilities.";

pub static CARD: CardDef = CardDef {
    abilities: &[
        // "When this enchantment enters, draw a card." (CR 603.6a)
        triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this enchantment enters, draw a card."),
        // "At the beginning of the end step, sacrifice this enchantment." Every end step, any player's (CR 513.1, 603.2); a
        // Dress Down cast with flash in an end step stays until the next one (CR 603.2b: the step has begun).
        triggered(EACH_END_STEP, effects::sacrifice_self, "At the beginning of the end step, sacrifice this enchantment."),
    ],
    statics: &[loses_all_abilities(all_creatures, TEXT)],
    ..oracle::DRESS_DOWN
};
