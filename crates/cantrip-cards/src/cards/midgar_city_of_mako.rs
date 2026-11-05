use cantrip_api::*;
use cantrip_mechanics::mana::enters_tapped;
use cantrip_mechanics::{effects, select};
use cantrip_oracle as oracle;

/// "You may sacrifice an artifact or creature. If you do, draw two cards." The sacrifice is chosen as the spell resolves and may be
/// declined (CR 608.2d, 701.21a); the cards are drawn only if a permanent was actually sacrificed (CR 608.2c).
fn sacrifice_to_draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::artifact_or_creature_permanents(select::up_to(1));
    if !effects::may_sacrifice(g, item.controller, item.object, rule)?.is_empty() {
        effects::draw(g, item, 2)?;
    }
    Ok(())
}

/// The Adventure (CR 715): You may sacrifice an artifact or creature. If you do, draw two cards.
pub static REACTOR_RAID: CardDef = CardDef { spell: spell(&[], sacrifice_to_draw_two), ..oracle::REACTOR_RAID };

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped),
    abilities: &[mana_ability(TAP, add(Mana::B), "{T}: Add {B}.")],
    adventure: Some(&REACTOR_RAID),
    ..oracle::MIDGAR_CITY_OF_MAKO
};

pub static EXTRA: &[&CardDef] = &[&REACTOR_RAID];
