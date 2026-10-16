use cantrip_api::*;
use cantrip_mechanics::empower;
use cantrip_oracle as oracle;

/// "Until end of turn, you may activate loyalty abilities of Jace planeswalkers you control on any player's turn any time you
/// could cast an instant. Empower Jace 8." The permission is a rule about the player (CR 606.3, 611.2c), so it reaches the
/// Jace token empower makes and any Jace that arrives later this turn; one loyalty ability per permanent each turn still
/// holds. A planeswalker type is not a word a text-changing effect changes.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::InstantLoyalty(Some(Subtype::Jace)), Duration::EndOfTurn);
    empower::jace(g, item.controller, item.object, 8)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::JACES_MACHINATIONS
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
