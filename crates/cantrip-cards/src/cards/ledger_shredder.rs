use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{connive, triggers};

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    connive(g, item.object, item.object)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::A_PLAYER_CASTS_THEIR_SECOND_SPELL, effect, "Whenever a player casts their second spell each turn, this creature connives. (Draw a card, then discard a card. If you discarded a nonland card, put a +1/+1 counter on this creature.)")],
    ..oracle::LEDGER_SHREDDER
};
