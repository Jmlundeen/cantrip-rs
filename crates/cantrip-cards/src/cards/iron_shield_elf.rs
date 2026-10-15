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
        g.add_effect(elf, elf, EffectKind::AddKeywords(Keywords::of(&[Keyword::Indestructible])), Duration::EndOfTurn);
        g.tap(elf);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Discard a card: This creature gains indestructible until end of turn. Tap it.",
        cost: Cost { discard: Some((1, any_card)), ..Cost::NONE },
        effect: shield,
        ..ActivatedDef::BLANK
    })],
    ..oracle::IRON_SHIELD_ELF
};
