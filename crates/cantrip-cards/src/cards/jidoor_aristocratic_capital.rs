use cantrip_api::*;
use cantrip_mechanics::mana::enters_tapped;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target opponent mills half their library, rounded down" (CR 701.17a, 107.1a): half of the library as the spell resolves, so
/// cards drawn or added in response count; nothing if the opponent is no longer a legal target (CR 608.2b).
fn mill_half_rounded_down(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let half = g.state().zone(Zone::library(opponent)).len() as u32 / 2;
    g.mill(opponent, half);
    Ok(())
}

/// The Adventure (CR 715): Target opponent mills half their library, rounded down.
pub static OVERTURE: CardDef = CardDef { spell: spell(&[targets::OPPONENT], mill_half_rounded_down), ..oracle::OVERTURE };

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped),
    abilities: &[mana_ability(TAP, add(Mana::U), "{T}: Add {U}.")],
    adventure: Some(&OVERTURE),
    ..oracle::JIDOOR_ARISTOCRATIC_CAPITAL
};

pub static EXTRA: &[&CardDef] = &[&OVERTURE];
