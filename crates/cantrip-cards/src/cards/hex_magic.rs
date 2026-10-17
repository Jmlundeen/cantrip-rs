use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Exile all the cards from your hand, then draw that many cards. Until the end of your next turn, you may play cards exiled
/// this way." The hand leaves together (CR 406.2, 101.4), face up; "that many" is the number of cards that really reached exile
/// (a replacement may send one elsewhere), and each of those may be played - cast, or played as the land for the turn
/// (CR 305.1) - until the cleanup step of its controller's next turn (CR 514.2). An empty hand exiles nothing and draws nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let grant = PlayGrant { lands: true, expiry: Expiry::EndOfYourNextTurn, ..PlayGrant::CAST };
    let mut exiled = 0;
    for card in g.exile_all(&hand) {
        if g.obj(card).zone.is(ZoneKind::Exile) {
            g.allow_play(card, you, grant);
            exiled += 1;
        }
    }
    g.draw(you, exiled);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::HEX_MAGIC };
