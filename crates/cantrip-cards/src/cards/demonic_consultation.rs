use cantrip_api::*;
use cantrip_mechanics::library::card_has_name;
use cantrip_oracle as oracle;

/// Choose a card name. Exile the top six cards of your library, then reveal cards from the top until you reveal a card with the chosen
/// name (CR 201.3, 701.20). Put that card into your hand and exile all other cards revealed this way, together; a miss
/// reveals and exiles what is left. Never a shuffle.
fn consult(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    // Choose a card name first: the instructions are followed in order (CR 608.2c).
    let Answer::Name(Some(wanted)) = g.ask(you, RequestKind::ChooseName { source: item.object, looked_at: Vec::new(), suggestion: None, nonland_only: false })? else {
        return Ok(());
    };
    let top: Vec<ObjectId> = g.state().zone(Zone::library(you)).iter().take(6).copied().collect();
    g.exile_all(&top);
    let library: Vec<ObjectId> = g.state().zone(Zone::library(you)).to_vec();
    let mut revealed = Vec::new();
    let mut found = None;
    for card in library {
        revealed.push(card);
        if card_has_name(g, g.chars(card).card, wanted) {
            found = Some(card);
            break;
        }
    }
    if revealed.is_empty() {
        return Ok(());
    }
    g.reveal(you, &revealed);
    let moves: Vec<(ObjectId, Zone)> =
        revealed.iter().map(|&c| (c, if Some(c) == found { Zone::hand(you) } else { Zone::exile(you) })).collect();
    g.move_together(&moves, ZoneReason::Other);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect: consult }),
    ..oracle::DEMONIC_CONSULTATION
};
