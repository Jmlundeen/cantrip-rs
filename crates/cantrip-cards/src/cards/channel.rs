use cantrip_api::*;
use cantrip_oracle as oracle;

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
    delayed: &[mana_ability(life(1), add(Mana::C), "Pay 1 life: Add {C}.")],
    ..oracle::CHANNEL
};
