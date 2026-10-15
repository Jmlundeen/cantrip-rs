use cantrip_api::*;
use cantrip_mechanics::combat::choose_attack_target;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// Reveal cards from the top of your library until you reveal a creature card. Put that card onto the battlefield
/// tapped and attacking and the rest on the bottom of your library in a random order.
fn reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let library: Vec<ObjectId> = g.state().zone(Zone::library(you)).to_vec();
    let mut revealed: Vec<ObjectId> = Vec::new();
    let mut creature: Option<ObjectId> = None;
    for &card in &library {
        revealed.push(card);
        if g.chars(card).is(CardType::Creature) {
            creature = Some(card);
            break;
        }
    }
    if revealed.is_empty() {
        return Ok(());
    }
    g.reveal(you, &revealed);
    if let Some(card) = creature {
        revealed.pop();
        let defender = choose_attack_target(g, you, item.object)?;
        match defender {
            Some(d) => {
                g.put_onto_battlefield_attacking(card, you, d)?;
            }
            None => {
                g.put_onto_battlefield(card, you, true)?;
            }
        }
    }
    g.bottom_in_random_order(you, &revealed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever Raph & Mikey attack, reveal cards from the top of your library until you reveal a creature card. Put that card onto the battlefield tapped and attacking and the rest on the bottom of your library in a random order.",
        listens: EventMask::ATTACKS,
        matches: triggers::this_attacks,
        effect: reveal,
        ..TriggeredDef::BLANK
    })],
    ..oracle::RAPH_MIKEY_TROUBLEMAKERS
};
