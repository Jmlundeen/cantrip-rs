use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{DEALS_COMBAT_DAMAGE_TO_PLAYER, THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Create a Blood token."
fn blood(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_blood(g, item.controller)?;
    Ok(())
}

fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, blood, "When Ivora enters and whenever it deals combat damage to a player, create a Blood token."),
        triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, blood, "When Ivora enters and whenever it deals combat damage to a player, create a Blood token."),
        triggered(cantrip_mechanics::triggers::YOU_DISCARD, counter, "Whenever you discard a card, put a +1/+1 counter on Ivora."),
    ],
    ..oracle::IVORA_INSATIABLE_HEIR
};

pub static EXTRA: &[&CardDef] = &[&tokens::BLOOD];
