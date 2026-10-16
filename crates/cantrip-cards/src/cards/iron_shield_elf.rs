use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn any_card(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

/// This creature gains indestructible until end of turn (a layer 6 keyword, CR 702.12b, 611.2a), then it is tapped.
/// The tap is the ability's own second instruction (CR 701.26a), not part of its cost, so the ability can be
/// activated while the Elf is already tapped.
fn shield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let elf = item.object;
    if targets::permanent(g, elf) {
        cantrip_mechanics::effects::gain_indestructible_until_end_of_turn(g, elf, elf);
        g.tap(elf);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(discard(1, any_card), shield, "Discard a card: This creature gains indestructible until end of turn. Tap it.")],
    ..oracle::IRON_SHIELD_ELF
};
