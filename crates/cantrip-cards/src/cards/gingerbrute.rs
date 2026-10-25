use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "{1}: This creature can't be blocked this turn except by creatures with haste." A resolved effect on this creature, not an
/// ability it gains (CR 113.1a, 611.2c): it lasts the turn whatever abilities the creature loses afterwards (CR 613.1f), and
/// it ends if the creature leaves the battlefield (CR 400.7).
fn cant_be_blocked_this_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::BlockRestriction { card: item.ability.card, index: 0 }, Duration::EndOfTurn);
    Ok(())
}

/// The restriction on the creature `on` (CR 509.1b): as an attacker, only a creature with haste as blockers are declared may
/// block it. It says nothing of its blocking, nor of a block already made (its ruling).
fn except_by_haste(q: &dyn Query, on: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    on != attacker || q.chars(blocker).has(Keyword::Haste)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}"), cant_be_blocked_this_turn, "{1}: This creature can't be blocked this turn except by creatures with haste."),
        ability(
            cost("{2}, {T}").and(SACRIFICE_THIS),
            |g, item| effects::gain_life(g, item, 3),
            "{2}, {T}, Sacrifice this creature: You gain 3 life.",
        ),
    ],
    lasting_block_rules: &[LastingBlockRule { text: "Can't be blocked except by creatures with haste", allows: except_by_haste }],
    ..oracle::GINGERBRUTE
};
