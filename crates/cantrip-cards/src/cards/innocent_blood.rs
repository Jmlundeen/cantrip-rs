use cantrip_api::*;
use cantrip_mechanics::each_player_sacrifices;
use cantrip_oracle as oracle;

/// Each player sacrifices a creature of their choice. Every player, the caster included, chooses in turn order and the
/// creatures are sacrificed together (CR 101.4, 701.21a); a player with no creature sacrifices nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_player_sacrifices(g, item.object, 1, |q, id| q.chars(id).is(CardType::Creature))?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::INNOCENT_BLOOD };
