use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this enchantment enters, you get {E}{E}" (CR 603.6a, 107.14).
fn energy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.change_energy(item.controller, 2);
    Ok(())
}

/// "Creature spells with mana value 3 or less" that you cast, judged on the face cast (a creature's instant Adventure is
/// no creature spell, CR 715.3a).
fn small_creature_spells_you_cast(q: &dyn Query, asker: &Asker, spell: ObjectId, player: PlayerId) -> bool {
    player == asker.controller && q.chars(spell).is(CardType::Creature) && q.mana_value(spell) <= 3
}

/// "You may cast creature spells with mana value 3 or less by paying {E} rather than paying their mana costs. If you cast a
/// spell this way, you may cast it as though it had flash." An alternative cost and no permission (CR 118.9, 601.3): it is
/// paid on a cast from wherever the card may already be cast from, X is 0 (CR 107.3b), additional costs are still paid
/// (CR 118.9d), and the flash is that cast's alone.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, energy, "When this enchantment enters, you get {E}{E} (two energy counters).")],
    spell_alternatives: &[SpellAlternative {
        text: "Pay {E} rather than its mana cost",
        applies: small_creature_spells_you_cast,
        cost: InsteadOfManaCost::Energy(1),
        flash: true,
    }],
    ..oracle::PRIMAL_PRAYERS
};
