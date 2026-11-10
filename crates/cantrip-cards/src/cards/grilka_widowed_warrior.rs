use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever this creature deals combat damage to a player, create that many tapped Treasure tokens.": "that many" is the
/// combat damage dealt (CR 510.2), which `DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT` carries as its first number (the plain
/// trigger fires without one). The Treasures come from the ability's
/// controller, which the granted ability's controller is (CR 109.5, 603.3a).
fn make_treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_by_rule(g, item.controller, &tokens::TREASURE, item.data[0] as u32, true)?;
    Ok(())
}

/// "Gain control of target creature an opponent controls until end of turn. Untap it. Until end of turn, it gains trample,
/// haste, and '...'". Control and untap and haste are Act of Treason's (CR 613.1b, 611.2a, 702.10); trample and the
/// granted triggered ability are fixed to the creature as the ability resolves (CR 611.2c), the ability in layer 6 (CR
/// 613.1f), so it is the stolen creature's own and its controller creates the Treasure. A target that has become illegal
/// is no creature to take (CR 608.2b).
fn steal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = effects::gain_control_untap_haste(g, item) else { return Ok(()) };
    // Each sentence is judged before the next (CR 603.8, 608.2c): the untap and haste the helper ends are its own.
    g.end_sentence();
    cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
    g.end_sentence();
    let this = g.card_id(&CARD);
    g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
    g.end_sentence();
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        steal,
        "When Grilka enters, gain control of target creature an opponent controls until end of turn. Untap it. Until end of turn, it gains trample, haste, and \"Whenever this creature deals combat damage to a player, create that many tapped Treasure tokens.\"",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    delayed: &[triggered(
        triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT,
        make_treasure,
        "Whenever this creature deals combat damage to a player, create that many tapped Treasure tokens.",
    )],
    ..oracle::GRILKA_WIDOWED_WARRIOR
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
