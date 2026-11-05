use cantrip_api::*;
use cantrip_mechanics::{choose, top_of_library, triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT};
use cantrip_oracle as oracle;

/// "Exile the top X cards of their library, where X is the amount of damage dealt. You may cast any number of spells with mana
/// value X or less from among them without paying their mana costs." (CR 406, 608.2g, 118.9, 107.3) The damaged player is the
/// trigger's subject and the damage its data. The cards leave together (a replacement may send one elsewhere, so only those that
/// reached exile are offered), and Kotis's controller is offered the castable ones of mana value X or less one at a time; a land
/// is not a spell, and a spell with X in its cost is cast with X as zero.
fn exile_and_cast_free(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(victim) = item.subject else { return Ok(()) };
    let x = item.data[0];
    let top = top_of_library(g, victim, x as usize);
    let exiled: Vec<ObjectId> = g.exile_all(&top).into_iter().filter(|&c| g.obj(c).zone.is(ZoneKind::Exile) && g.mana_value(c) <= x).collect();
    choose::cast_any_number_free(g, item.controller, item.object, &exiled)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT,
        exile_and_cast_free,
        "Whenever Kotis deals combat damage to a player, exile the top X cards of their library, where X is the amount of damage dealt. You may cast any number of spells with mana value X or less from among them without paying their mana costs.",
    )],
    ..oracle::KOTIS_THE_FANGKEEPER
};
