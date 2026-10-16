//! Plot (CR 702.170a): "Any time you have priority during your main phase while the stack is empty, you may exile this card
//! from your hand and pay [cost]. It becomes a plotted card. Cast it as a sorcery on a later turn without paying its mana
//! cost." Pillage the Bog and Pitiless Carnage print it.

use cantrip_api::*;

/// Exile the card from the hand and let its owner cast it for free as a sorcery on a later turn (CR 702.170d).
fn plot_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    let owner = g.obj(card).owner;
    let exiled = g.exile(card);
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.allow_play(
            exiled,
            owner,
            PlayGrant { instead: Some(ManaCost::ZERO), later_turns: true, sorcery_speed: true, ..PlayGrant::CAST },
        );
    }
    Ok(())
}

/// Plot [cost]: a special action (CR 116.2k) from the hand, at sorcery timing. `text` is the ability's text, which a
/// scenario's `Ability:` matches.
pub const fn plot(cost: ManaCost, text: &'static str) -> AbilityDef {
    ability(Cost::mana(cost), plot_it, text)
        .sorcery_speed()
        .from(ZoneKind::Hand)
        .special_action()
}
