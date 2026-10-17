use cantrip_api::*;
use cantrip_mechanics::{hand::discard_chosen_nonland_from_revealed_hand, targets};
use cantrip_oracle as oracle;

/// Target opponent reveals their hand; you choose a nonland card from it; they discard it (Duress's shape). Then put a
/// +1/+1 counter on a creature you control, chosen as the spell resolves rather than targeted (CR 608.2d): a lone
/// creature is taken without asking, and with none the sentence does nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(opponent) = item.targets[0].player() {
        discard_chosen_nonland_from_revealed_hand(g, item, opponent)?;
    }
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let chosen = match creatures.len() {
        0 => None,
        1 => Some(creatures[0]),
        _ => {
            let ask = RequestKind::ChooseTarget { label: "", slot_player: None, total: None,
                source: item.object,
                purpose: Purpose::Targets,
                candidates: creatures.iter().map(|&id| Target::Object(id)).collect(),
                optional: false,
                least: 1,
                count: 1,
                any_number: false,
            };
            match g.ask(item.controller, ask)? {
                Answer::Targets(t) => t.first().and_then(|t| t.object()),
                _ => None,
            }
        }
    };
    if let Some(creature) = chosen {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::HUMILIATE
};
