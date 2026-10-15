use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "Your team may sacrifice any number of creatures. For each creature sacrificed this way, you add {B}{B}." There are
/// no teams here, so the team is the caster alone: they choose any number of their creatures, all are sacrificed at
/// once (CR 701.21a), and the mana counts those that actually were.
fn harvest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let eligible: Vec<ObjectId> = g.permanents_of(you).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let chosen = choose::cards(g, you, item.object, Purpose::Sacrifice, eligible, u8::MAX, true)?;
    if chosen.is_empty() {
        return Ok(());
    }
    let sacrificed = g.sacrifice(&chosen).len();
    if sacrificed > 0 {
        g.add_mana(you, Color::Black.mana(), (2 * sacrificed).min(255) as u8);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect: harvest }), ..oracle::ORGAN_HARVEST };
