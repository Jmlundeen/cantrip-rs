use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Players can't gain life this turn. Damage can't be prevented this turn. Skullcrack deals 3 damage to target player or
/// planeswalker." In that order (CR 608.2c): each player in the game can't gain life until the turn ends (CR 119.7), so a
/// replacement of a gain does not apply either (the rulings); no damage can be prevented this turn, its own 3 included
/// (CR 615.12) - a prevention effect still resolves and prevents nothing. A target that is no longer legal stops the whole
/// spell (CR 608.2b, the ruling): the core does not resolve it.
fn skullcrack(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    for p in players {
        g.add_effect(item.object, item.object, EffectKind::CantGainLife(p), Duration::EndOfTurn);
    }
    g.damage_cant_be_prevented();
    damage_target(g, item, 0, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::PLAYER_OR_PLANESWALKER], skullcrack), ..oracle::SKULLCRACK };
