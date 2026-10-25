use cantrip_api::*;
use cantrip_mechanics::{choose, search_library};
use cantrip_oracle as oracle;

const ENTER: Question = Question::new(
    "PutOntoBattlefield",
    "pay {cost} to put {subject} onto the battlefield?",
    "If you don't, it goes to the graveyard",
    "Pay {cost}",
    "Put it in the graveyard",
);

/// Sacrifice an artifact; if you do, search for an artifact card. At most the sacrificed artifact's mana value it
/// enters; above, you may pay the difference to put it onto the battlefield, else it goes to its owner's graveyard.
/// Then shuffle, even when nothing was sacrificed.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let artifacts: Vec<ObjectId> =
        g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Artifact)).collect();
    if let Some(sacrificed) = choose::card(g, player, item.object, Purpose::Sacrifice, artifacts)? {
        let value = g.mana_value(sacrificed);
        if !g.sacrifice(player, &[sacrificed]).is_empty() {
            let found = search_library(g, player, player, item.object, SearchDestination::Found, 1, true, |q, c| q.chars(c).is(CardType::Artifact))?;
            if let Some(&card) = found.first() {
                let cost = g.mana_value(card);
                let enter = if cost <= value {
                    true
                } else {
                    let diff = generic((cost - value) as u8);
                    g.may_pay(player, item.object, card, of_mana(diff), &ENTER)?
                };
                if enter {
                    g.put_onto_battlefield(card, player, false)?;
                } else {
                    let owner = g.obj(card).owner;
                    g.move_together(&[(card, Zone::graveyard(owner))], MoveReason::Search);
                }
            }
        }
    }
    g.shuffle(player);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::TRANSMUTE_ARTIFACT };
