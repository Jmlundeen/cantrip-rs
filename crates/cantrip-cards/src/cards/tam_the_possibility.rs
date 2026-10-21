use std::collections::BTreeSet;

use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Planeswalker spells you cast cost {1} less to cast" (CR 601.2f): only its controller's spells, judged on the spell's
/// own characteristics as it is cast, and only a planeswalker spell. The reduction is generic (CR 118.7a), so a cost with
/// no generic mana is unchanged.
fn planeswalker_spells_cost_one_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    if q.chars(spell).is(CardType::Planeswalker) { CostDelta::less(1) } else { CostDelta::NONE }
}

/// "{W}{U}{B}{R}{G}, {T}: Proliferate X times, where X is the number of planeswalker types among planeswalkers you
/// control." (CR 701.34a, 205.3j, 306.3): each distinct planeswalker type among the planeswalkers its controller controls
/// as the ability resolves, counted once however many planeswalkers have it. X is fixed as the ability resolves (CR 608.2c),
/// then that many separate proliferate actions happen, one after another: they are one effect resolving, so no player
/// receives priority between them.
fn proliferate_for_each_planeswalker_type(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let mut types: BTreeSet<Subtype> = BTreeSet::new();
    for &id in g.state().battlefield() {
        if g.controller(id) != you {
            continue;
        }
        let chars = g.chars(id);
        if !chars.is(CardType::Planeswalker) {
            continue;
        }
        // A planeswalker's current subtypes are its planeswalker types (CR 306.3, 205.3j). Layer 4 may change those
        // characteristics (CR 613.1d); other subtypes, such as a creature type added to an animated Gideon, are not
        // planeswalker types and do not count.
        types.extend(chars.subtypes.iter().filter(|subtype| subtype.card_types().has(CardType::Planeswalker)));
    }
    for _ in 0..types.len() {
        g.proliferate(you, item.object)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[planeswalker_spells_cost_one_less],
    abilities: &[ability(
        cost("{W}{U}{B}{R}{G}").and(TAP),
        proliferate_for_each_planeswalker_type,
        "{W}{U}{B}{R}{G}, {T}: Proliferate X times, where X is the number of planeswalker types among planeswalkers you control.",
    )],
    ..oracle::TAM_THE_POSSIBILITY
};
