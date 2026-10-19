use cantrip_api::*;
use cantrip_mechanics::creatures_you_control;
use cantrip_oracle as oracle;

/// "You may cast spells with mana value less than or equal to the number of creatures you control from your
/// hand without paying their mana costs" Compare current creature count with the spell’s mana value as it is
/// cast.
fn free_spells_from_hand(q: &dyn Query, asker: &Asker, spell: ObjectId, player: PlayerId) -> bool {
    player == asker.controller
        && q.obj(spell).zone.is(ZoneKind::Hand)
        && q.mana_value(spell) as usize <= creatures_you_control(q, player)
}

pub static CARD: CardDef = CardDef {
    spell_alternatives: &[SpellAlternative {
        text: "Without paying its mana cost",
        applies: free_spells_from_hand,
        cost: InsteadOfManaCost::WithoutPaying,
    }],
    ..oracle::OMNIPRESENCE
};
