use cantrip_api::*;
use cantrip_mechanics::effects::{self, each_opponent_discards_a_card};
use cantrip_mechanics::triggers::{THIS_DIES, THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this enchantment enters, each opponent discards a card and loses 2 life." The discard is the first sentence of
/// the instruction, the life loss follows whether or not anyone could discard (ruling 2023-09-01; CR 608.2c).
fn discard_and_lose_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_discards_a_card(g, item)?;
    g.end_sentence();
    g.each_opponent_loses_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, discard_and_lose_life, "When this enchantment enters, each opponent discards a card and loses 2 life."),
        // Sacrificed or destroyed, it is looked back on (CR 603.10a) and scries from the graveyard.
        triggered(THIS_DIES, effects::scry_two, "When this enchantment is put into a graveyard from the battlefield, scry 2."),
        ability(cost("{2}{B}"), effects::sacrifice_self, "{2}{B}: Sacrifice this enchantment."),
    ],
    ..oracle::HOPELESS_NIGHTMARE
};
