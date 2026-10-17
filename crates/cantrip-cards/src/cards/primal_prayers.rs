use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this enchantment enters, you get {E}{E}" (CR 603.6a, 107.14).
fn energy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.change_energy(item.controller, 2);
    Ok(())
}

/// "You may cast creature spells with mana value 3 or less by paying {E} rather than paying their mana costs."
/// The core reads the proposed face and owns alternative costs, X = 0, additional costs and timing
/// (CR 118.9, 107.3b, 601.3c); this text supplies only its controller, type and mana-value restriction.
fn for_energy(q: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> Option<Cost> {
    (player == asker.controller && q.chars(card).is(CardType::Creature) && q.mana_value(card) <= 3)
        .then_some(Cost { energy: 1, ..Cost::NONE })
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, energy, "When this enchantment enters, you get {E}{E} (two energy counters).")],
    granted_ways: &[GrantedWay {
        text: "Pay {E} rather than its mana cost",
        cost: for_energy,
        from: ZoneKind::Hand,
        any_zone: true,
        instant_speed: true,
        ..GrantedWay::CAST
    }],
    ..oracle::PRIMAL_PRAYERS
};
