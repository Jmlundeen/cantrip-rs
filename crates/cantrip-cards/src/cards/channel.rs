use cantrip_api::*;
use cantrip_oracle as oracle;

/// The mana ability Channel gives its controller: "Pay 1 life: Add {C}." (CR 605.1a.) The player's own, not the card's, so
/// it is not among the card's `abilities`; it is the card's one `delayed` ability, which the effect it has granted names.
fn add_colorless(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::C, 1);
    Ok(())
}

/// "Until end of turn, any time you could activate a mana ability, you may pay 1 life. If you do, add {C}." The player is
/// given a mana ability (CR 611.2a, 605.1a) with no source permanent, whose cost is paying 1 life (CR 119.4): activated
/// whenever they could activate any mana ability, including while paying for a spell, as often as they have life to pay. It
/// is the player's, so it outlasts the card wherever that goes, and a copy that has ceased to exist.
fn grant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = g.obj(item.object).card;
    g.add_effect(item.object, item.object, EffectKind::PlayerManaAbility(AbilityRef::delayed(card, 0)), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect: grant }),
    delayed: &[AbilityDef::Activated(ActivatedDef {
        text: "Pay 1 life: Add {C}.",
        cost: Cost { life: 1, ..Cost::NONE },
        mana: Some(add_colorless),
        adds: ManaPool([0, 0, 0, 0, 0, 1]),
        ..ActivatedDef::BLANK
    })],
    ..oracle::CHANNEL
};
