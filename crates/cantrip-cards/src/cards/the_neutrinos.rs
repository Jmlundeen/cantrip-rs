use cantrip_api::*;
use cantrip_mechanics::combat::choose_attack_target;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Exile up to one target creature you own, then return it to the battlefield under your control tapped and attacking" (CR 400.7,
/// 508.4): it comes back as a new object under the ability's controller, tapped, attacking a player or planeswalker its controller
/// chooses, which need not be what The Neutrinos attack; it was never declared as an attacker, so nothing that watches attackers
/// being declared sees it. A token that is exiled ceases to exist and does not return (CR 111.7). With nothing it can attack, it
/// just returns tapped. Nothing happens if the creature has left the battlefield by now (CR 608.2b). Each card of a merged
/// permanent returns (CR 730.3c), together and tapped, each attacking what its controller chooses (CR 508.4) - a token among the
/// components has ceased to exist.
fn blink_tapped_and_attacking(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let cards: Vec<ObjectId> = g.exile_all(&[creature]).into_iter().filter(|&c| !g.obj(c).token && g.obj(c).zone.is(ZoneKind::Exile)).collect();
    let you = item.controller;
    // What each attacks is chosen for that creature (CR 508.4), paired with it by object: a card kept from entering has none.
    let mut defenders = Vec::new();
    for &card in &cards {
        defenders.push((card, choose_attack_target(g, you, card)?.unwrap_or(Target::None)));
    }
    let entering: Vec<(ObjectId, PlayerId, bool)> = cards.iter().map(|&c| (c, you, true)).collect();
    let entered = g.put_all_onto_battlefield_as(&entering)?;
    for (card, defender) in defenders {
        // The permanent a card became as it entered: its `became` names it.
        if let Some(&permanent) = entered.iter().find(|&&e| g.obj(card).became == Some(e)) {
            g.enter_attacking(&[permanent], defender);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS,
            effects::this_gets::<1, 0>,
            "Alliance \u{2014} Whenever another creature you control enters, The Neutrinos get +1/+0 until end of turn.",
        ),
        triggered(
            triggers::THIS_ATTACKS,
            blink_tapped_and_attacking,
            "Whenever The Neutrinos attack, exile up to one target creature you own, then return it to the battlefield under your control tapped and attacking.",
        )
        .targets(&[up_to_one(targets::CREATURE_YOU_OWN.filter)]),
    ],
    ..oracle::THE_NEUTRINOS
};
