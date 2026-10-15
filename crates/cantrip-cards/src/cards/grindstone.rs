use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Target player mills two cards; if two cards that share a color were milled this way, repeat this process. Each pass
/// is one simultaneous mill of the top two cards (CR 701.13a), and only the cards that reached the graveyard count as
/// milled: a pass with fewer than two cannot share a color, and colorless cards share none (CR 105.2a). The pass that
/// does not shorten the library stops the loop rather than spinning.
fn grind(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(who) = item.targets[0].player() else { return Ok(()) };
    loop {
        let library = g.state().zone(Zone::library(who)).to_vec();
        if library.is_empty() {
            return Ok(());
        }
        let milled: Vec<ObjectId> = g.mill(who, 2).into_iter().filter(|&c| g.obj(c).zone.is(ZoneKind::Graveyard)).collect();
        if milled.len() < 2 || !g.chars(milled[0]).colors.intersects(g.chars(milled[1]).colors) {
            return Ok(());
        }
        if g.state().zone(Zone::library(who)).len() >= library.len() {
            return Ok(());
        }
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{3}, {T}: Target player mills two cards. If two cards that share a color were milled this way, repeat this process.",
        cost: Cost { mana: mana(3, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
        targets: &[targets::PLAYER],
        effect: grind,
        ..ActivatedDef::BLANK
    })],
    ..oracle::GRINDSTONE
};
