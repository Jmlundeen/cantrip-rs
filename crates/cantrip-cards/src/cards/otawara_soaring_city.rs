use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::{channel, mana};
use cantrip_oracle as oracle;

/// "Target artifact, creature, enchantment, or planeswalker" (CR 115.1): on the battlefield, whoever controls it.
fn bounceable(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::PERMANENT.filter)(q, asker, t) && matches!(t, Target::Object(id) if {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature) || c.is(CardType::Enchantment) || c.is(CardType::Planeswalker)
    })
}

/// Channel: return the target to its owner's hand.
fn bounce(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.return_to_hand(&[permanent]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_u, Mana::U, "{T}: Add {U}."),
        AbilityDef::Activated(ActivatedDef {
            text: "Channel \u{2014} {3}{U}, Discard this card: Return target artifact, creature, enchantment, or planeswalker to its owner's hand. This ability costs {1} less to activate for each legendary creature you control.",
            cost: Cost { mana: mana(3, 0, 1, 0, 0, 0), discard_self: true, ..Cost::NONE },
            targets: &[TargetSpec { min: 1, max: 1, filter: bounceable }],
            effect: bounce,
            from: ZoneKind::Hand,
            ..ActivatedDef::BLANK
        }),
    ],
    own_cost_changes: &[channel::cheaper_per_legendary_creature],
    ..oracle::OTAWARA_SOARING_CITY
};
