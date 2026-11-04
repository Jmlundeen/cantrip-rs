use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Until end of turn, you may look at the top card of your library any time and you may play cards from the top of your
/// library. If you cast a spell this way, pay life equal to its mana value rather than pay its mana cost." (CR 611.2a, 401.2,
/// 305.1, 601.3, 118.9): the player is given Bolas's Citadel's two play rules for the turn, as effects of the resolved
/// ability. They are the player's, so they outlast Gwenom leaving the battlefield or losing its abilities. One instruction:
/// both are given at once.
fn look_and_play_from_the_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.ability.card;
    instruction(g, |g| {
        for index in 0..2 {
            g.add_effect(item.object, item.object, EffectKind::PlayerPlayRule(AbilityRef::delayed(card, index)), Duration::EndOfTurn);
        }
    });
    Ok(())
}

/// Deathtouch and lifelink are core keywords read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        look_and_play_from_the_top,
        "Whenever Gwenom attacks, until end of turn, you may look at the top card of your library any time and you may play cards from the top of your library. If you cast a spell this way, pay life equal to its mana value rather than pay its mana cost.",
    )],
    delayed: &[
        granted_play_rule(PlayRule::LookAtLibraryTop, "You may look at the top card of your library any time."),
        granted_play_rule(
            PlayRule::LibraryTop,
            "You may play cards from the top of your library. If you cast a spell this way, pay life equal to its mana value rather than pay its mana cost.",
        ),
    ],
    ..oracle::GWENOM_REMORSELESS
};
