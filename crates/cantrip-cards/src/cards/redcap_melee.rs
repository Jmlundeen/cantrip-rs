use cantrip_api::*;
use cantrip_mechanics::{sacrifice_chosen, targets};
use cantrip_oracle as oracle;

/// "Redcap Melee deals 4 damage to target creature or planeswalker. If a nonred permanent is dealt damage this way, you
/// sacrifice a land." Any permanent actually dealt damage by this spell counts, which a redirection (CR 614.1a, Ancient
/// Adamantoise) can make another than the target, or several (Harm's Way; redirected damage keeps this spell as its source,
/// CR 614.9): they are read from the damage events this resolution recorded (CR 120.3), so prevented damage names none. Nonred is read through the layers (CR 105.2), so a colorless one
/// counts, and "red" through the spell's words (CR 612). The land is the controller's choice (CR 701.21a).
fn melee(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let before = g.history().head() as usize;
    g.damage(item.object, item.target(0), 4);
    let red = g.words(item.object).color(Color::Red);
    // Damage to a player, or from another source, names no permanent this spell dealt damage to.
    let dealt_to = |r: &Record| match r.event { Event::Damage { source, target: Target::Object(id), amount, .. } if source == item.object && amount > 0 => Some(id), _ => None };
    let nonred_dealt = g.history().all()[before..].iter().filter_map(dealt_to).any(|id| !g.chars(id).colors.has(red));
    if nonred_dealt {
        sacrifice_chosen(g, item.controller, item.object, 1, |c| c.is(CardType::Land))?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], melee),
    ..oracle::REDCAP_MELEE
};
